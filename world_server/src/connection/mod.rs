//! Per-socket network handler for the world server.
//!
//! - Owns the raw `TcpStream`, encryption/decryption state and minimal per-connection data.
//! - Translates wire-level packets into higher-level `ClientEvent`s sent to the client manager;
//!   the manager owns gameplay/session state so networking stays dumb and testable.
//! - Uses a private `ServerEvent` channel so the manager can push outbound messages without
//!   holding a mutable reference to the connection (improves isolation & concurrency).
//! - Runs an event loop that races incoming client packets against manager-originated server
//!   events to avoid head-of-line blocking (slow client writing does not delay server pushes).
//! - Performs only the authentication handshake locally (seed + `CMSG_AUTH_SESSION`) because
//!   the handshake needs immediate access to cryptographic material bound to the transport.
//! - Keeps encryption halves optional until auth succeeds, making the state transition explicit.

pub mod events;

use std::sync::Arc;

use anyhow::Result;
use smol::net::TcpStream;
use tracing::*;
use wow_srp::wrath_header::ProofSeed;
use wow_srp::wrath_header::ServerDecrypterHalf;
use wow_srp::wrath_header::ServerEncrypterHalf;
use wow_world_messages::wrath::opcodes::ClientOpcodeMessage;
use wow_world_messages::wrath::CMSG_AUTH_SESSION;
use wow_world_messages::wrath::SMSG_AUTH_CHALLENGE;

use wow_world_messages::wrath::astd_expect_client_message;
use wrath_auth_db::AuthDatabase;

use crate::handlers::handle_cmsg_auth_session;
use crate::packet::ServerMessageExt;
use events::{ClientEvent, ConnectionEvent, ServerEvent};

pub struct ConnectionData {
    pub account_id: Option<u32>,
}

/// Network connection wrapper; isolates socket, crypto state and messaging glue.
pub struct Connection {
    pub stream: TcpStream,
    client_manager_sender: flume::Sender<ClientEvent>,

    // Used to send events from the client manager to this connection
    sender: flume::Sender<ServerEvent>,
    receiver: flume::Receiver<ServerEvent>,

    pub encryption: Option<ServerEncrypterHalf>,
    decryption: Option<ServerDecrypterHalf>,

    data: ConnectionData,
}

impl Connection {
    /// Construct a new connection; creates an internal channel for manager-driven outbound events.
    pub fn new(stream: TcpStream, client_manager_sender: flume::Sender<ClientEvent>) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            stream,
            client_manager_sender,
            sender,
            receiver,
            encryption: None,
            decryption: None,
            data: ConnectionData { account_id: None },
        }
    }

    /// Acquire a clone of the outbound server-event sender for registration with manager structures.
    pub fn get_sender(&self) -> flume::Sender<ServerEvent> {
        self.sender.clone()
    }

    /// Indicates whether the connection completed authentication (post `CMSG_AUTH_SESSION`).
    pub fn is_authenticated(&self) -> bool {
        self.data.account_id.is_some()
    }

    /// Install negotiated crypto; split halves allow send/recv to proceed independently.
    pub fn set_crypto(&mut self, encryption: ServerEncrypterHalf, decryption: ServerDecrypterHalf) {
        self.encryption.replace(encryption);
        self.decryption.replace(decryption);
    }

    /// Gracefully terminate: inform the manager so it can clean up any retained session state.
    pub async fn disconnect(&mut self) -> Result<()> {
        let addr = self.stream.local_addr().unwrap();
        info!("Disconnecting client {addr}");
        // Let the client manager know that this client is disconnecting
        self.client_manager_sender.send_async(ClientEvent::Disconnected { addr }).await?;
        Ok(())
    }

    /// Entry point for a newly accepted socket: run handshake + bidirectional event loop + teardown.
    pub async fn run(mut self, auth_db: Arc<AuthDatabase>) {
        let addr = self.stream.local_addr().unwrap();
        info!("New connection from {addr}");
        if let Err(e) = self.update(auth_db).await {
            error!("Error in client update {addr}: {e:?}");
        }
        self.disconnect().await.unwrap_or_else(|e| {
            error!("Error disconnecting client {addr}: {e:?}");
        });
    }

    /// Perform authentication then interleave reads from client and commands from manager until disconnect.
    pub async fn update(&mut self, auth_db: Arc<AuthDatabase>) -> Result<()> {
        // Authenticate first
        let proof_seed = ProofSeed::new();
        self.send_auth_challenge(&proof_seed).await?;

        let auth_session_packet = astd_expect_client_message::<CMSG_AUTH_SESSION, _>(&mut self.stream).await?;

        let account_id = handle_cmsg_auth_session(self, proof_seed, &auth_session_packet, auth_db).await?;

        // Then, advertise the new connection to the client manager
        let addr = self.stream.local_addr()?;
        let connection_event = ClientEvent::Connected {
            addr,
            account_id,
            connection_sender: self.sender.clone(),
        };
        self.client_manager_sender.send_async(connection_event).await?;

        // Then race between receiving from the client and receiving from the manager
        loop {
            let event = smol::future::race(
                receive_from_client(&mut self.stream, self.decryption.as_mut().unwrap()),
                receive_from_manager(&self.receiver),
            )
            .await?;

            match event {
                ConnectionEvent::Client(packet) => {
                    info!("Handling packet {packet} from client {addr}");
                    let client_message = ClientEvent::Message { addr, packet };
                    self.client_manager_sender.send_async(client_message).await?;
                }
                ConnectionEvent::Server(server_event) => {
                    info!("Sending {server_event} from server to client {addr}");
                    match server_event {
                        ServerEvent::AccountDataTimes(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::ActionButtons(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::BindPointUpdate(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::CalendarSendNumPending(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::CharCreate(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::CharDelete(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::CharEnum(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::ContactList(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::DestroyObject(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::FeatureSystemStatus(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::ForceMoveRoot(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::ForceMoveUnroot(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::GMTicketGetTicket(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::GMTicketSystemStatus(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::InitialSpells(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::InitializeFactions(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::InitWorldStates(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::ItemNameQueryResponse(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::ItemQuerySingleResponse(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::LoginVerifyWorld(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::LoginSetTimeSpeed(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::LogoutComplete(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::LogoutCancelAck(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::LogoutResponse(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MessageChat(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveFallLand(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveHeartbeat(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveJump(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveSetFacing(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveSetRunMode(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveSetWalkMode(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartBackward(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartForward(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartPitchDown(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartPitchUp(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartStrafeLeft(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartStrafeRight(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStop(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartSwim(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartTurnLeft(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStartTurnRight(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStopPitch(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStopStrafe(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStopSwim(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveStopTurn(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::MoveTeleportAck(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::NameQueryResponse(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::NewWorld(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::PlayedTime(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::WorldStateUiTimerUpdate(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::QueryTimeResponse(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::Pong(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::RaidInstanceInfo(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::RealmSplit(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::SetDungeonDifficulty(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::StandStateUpdate(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::TimeSyncReq(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::TransferPending(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::TriggerCinematic(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::TutorialFlags(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::UpdateAccountDataComplete(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::UpdateAccountData(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::UpdateObject(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::UpdateWorldState(m) => m.astd_send_to_connection(self).await?,
                        ServerEvent::Disconnect => {
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Send initial auth challenge; seeds later key derivation and establishes crypto context.
    pub async fn send_auth_challenge(&mut self, proof_seed: &ProofSeed) -> Result<()> {
        use wow_world_messages::wrath::ServerMessage;
        SMSG_AUTH_CHALLENGE {
            unknown1: 1,
            server_seed: proof_seed.seed(),
            seed: [0_u8; 32],
        }
        .astd_write_unencrypted_server(&mut self.stream)
        .await?;

        Ok(())
    }
}

/// Read & decrypt the next client packet; minimal framing logic kept near the transport boundary.
async fn receive_from_client(stream: &mut TcpStream, decrypter: &mut ServerDecrypterHalf) -> Result<ConnectionEvent> {
    let packet = ClientOpcodeMessage::astd_read_encrypted(stream, decrypter).await?;
    Ok(ConnectionEvent::Client(packet))
}

/// Await the next manager-originated server event for this connection.
async fn receive_from_manager(receiver: &flume::Receiver<ServerEvent>) -> Result<ConnectionEvent> {
    Ok(ConnectionEvent::Server(receiver.recv_async().await?))
}
