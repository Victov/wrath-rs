use super::character::*;
use super::client_manager::ClientManager;
use crate::handlers::handle_cmsg_auth_session;
use crate::handlers::login_handler::LogoutState;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use crate::world::World;
use smol::lock::{Mutex, RwLock};
use smol::net::TcpStream;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use wow_srp::wrath_header::ProofSeed;
use wow_srp::wrath_header::ServerDecrypterHalf;
use wow_srp::wrath_header::ServerEncrypterHalf;
use wow_world_messages::errors::ExpectedOpcodeError;
use wow_world_messages::wrath::astd_expect_client_message;
use wow_world_messages::wrath::opcodes::ClientOpcodeMessage;
use wow_world_messages::wrath::ServerMessage;
use wow_world_messages::wrath::CMSG_AUTH_SESSION;
use wow_world_messages::wrath::SMSG_AUTH_CHALLENGE;
use wow_world_messages::Guid;
use wrath_auth_db::AuthDatabase;

#[derive(Clone, PartialEq, Eq)]
pub enum ClientState {
    PreLogin,
    CharacterSelection,
    DisconnectPendingCleanup,
    Disconnected,
}

pub struct ClientData {
    pub client_state: ClientState,
    pub account_id: Option<u32>,
    pub active_character: Option<Arc<RwLock<Character>>>,
}

pub struct Client {
    pub id: u64,
    pub read_socket: Arc<RwLock<TcpStream>>,
    pub write_socket: Arc<Mutex<TcpStream>>,
    pub encryption: Mutex<Option<ServerEncrypterHalf>>,
    pub decryption: Mutex<Option<ServerDecrypterHalf>>,
    pub data: RwLock<ClientData>,
}

impl Client {
    pub fn new(id: u64, read_socket: Arc<RwLock<TcpStream>>, write_socket: Arc<Mutex<TcpStream>>) -> Self {
        Self {
            id,
            read_socket,
            write_socket,
            encryption: Mutex::new(None),
            decryption: Mutex::new(None),
            data: RwLock::new(ClientData {
                client_state: ClientState::PreLogin,
                account_id: None,
                active_character: None,
            }),
        }
    }

    pub async fn tick(&self, delta_time: f32, world: Arc<World>) -> Result<()> {
        let mut should_return_to_character_select: bool = false;
        if let Some(character_lock) = &self.data.read().await.active_character {
            let mut character = character_lock.write().await;
            character.tick(delta_time, world).await?;

            should_return_to_character_select = character.logout_state == LogoutState::ReturnToCharSelect;
        }

        if should_return_to_character_select {
            let data = &mut self.data.write().await;
            data.active_character = None;
            data.client_state = ClientState::CharacterSelection;
        }
        Ok(())
    }

    pub async fn authenticate_and_start_receiving_data(&self, packet_handle_sender: Sender<PacketToHandle>, auth_db: Arc<AuthDatabase>) {
        let proof_seed = ProofSeed::new();
        self.send_auth_challenge(&proof_seed).await.unwrap_or_else(|e| {
            error!("Error while sending auth challenge: {:?}", e);
        });

        let auth_session_packet = {
            let mut read_socket = self.read_socket.write().await;
            astd_expect_client_message::<CMSG_AUTH_SESSION, _>(&mut *read_socket).await
        };

        if let Ok(auth_session_packet) = auth_session_packet {
            handle_cmsg_auth_session(self, proof_seed, &auth_session_packet, auth_db)
                .await
                .unwrap_or_else(|e| {
                    warn!("Error while authenticating {:?}", e);
                });

            self.handle_incoming_packets(packet_handle_sender).await.unwrap_or_else(|e| {
                warn!("Error while handling packet {:?}", e);
            });
        }

        //Client stopped handling packets. Probably disconnected. Remove from client list?
        self.disconnect()
            .await
            .unwrap_or_else(|e| warn!("Something went wrong while disconnecting client: {:?}", e));
    }

    async fn handle_incoming_packets(&self, packet_channel: Sender<PacketToHandle>) -> Result<()> {
        loop {
            if !self.is_authenticated().await {
                break;
            }
            let opcode = {
                let decryptor_opt: &mut Option<ServerDecrypterHalf> = &mut *self.decryption.lock().await;
                if let Some(decryption) = decryptor_opt.as_mut() {
                    let mut read_socket = self.read_socket.write().await;
                    ClientOpcodeMessage::astd_read_encrypted(&mut *read_socket, decryption).await
                } else {
                    bail!("Encryption didn't exist");
                }
            };
            if let Ok(op) = opcode {
                packet_channel.send(PacketToHandle {
                    client_id: self.id,
                    payload: Box::new(op),
                })?;
            } else if let Err(e) = opcode {
                if let ExpectedOpcodeError::Io(_) = e {
                    error!("IO error during parsing, there is no recovery from this, disconnect client");
                    break;
                }
                warn!("Error in opcode: {}.", e);
            }
        }
        Ok(())
    }

    pub async fn get_active_character(&self) -> Result<Arc<RwLock<Character>>> {
        let data = self.data.read().await;
        let arc = data.active_character.as_ref().ok_or_else(|| anyhow!("No active character"))?;
        Ok(arc.clone())
    }

    pub async fn disconnect(&self) -> Result<()> {
        //Kill all networking, but allow the world one frame to do cleanup
        //For example, keep around the active character, so that the instance manager can see that
        //we're disconnected, but still access the character to do world cleanup
        info!("A client disconnected");

        {
            let data = &mut self.data.write().await;
            data.client_state = ClientState::DisconnectPendingCleanup;
        }
        self.read_socket.write().await.shutdown(smol::net::Shutdown::Both)?;
        self.write_socket.lock().await.shutdown(std::net::Shutdown::Both)?;
        //Save character to db?
        Ok(())
    }

    pub async fn disconnected_post_cleanup(&self) -> Result<()> {
        //Cleanup time has passed. Now this client is really really disconnected and
        //will be fully removed from memory
        let data = &mut self.data.write().await;
        data.client_state = ClientState::Disconnected;
        data.account_id = None;
        data.active_character = None;
        Ok(())
    }

    pub async fn send_auth_challenge(&self, proof_seed: &ProofSeed) -> Result<()> {
        let mut stream = self.write_socket.lock().await;
        SMSG_AUTH_CHALLENGE {
            unknown1: 1,
            server_seed: proof_seed.seed(),
            seed: [0_u8; 32],
        }
        .astd_write_unencrypted_server(&mut *stream)
        .await
        .unwrap();

        Ok(())
    }

    pub async fn is_authenticated(&self) -> bool {
        let data = self.data.read().await;
        data.account_id.is_some() && data.client_state != ClientState::PreLogin
    }

    pub async fn load_and_set_active_character(&self, client_manager: &ClientManager, world: &World, character_guid: Guid) -> Result<()> {
        let weakself = Arc::downgrade(&client_manager.get_client(self.id).await?);
        let character = Character::load(weakself, character_guid, world, &client_manager.data_storage).await?;
        let character_arc = Arc::new(RwLock::new(character));

        let mut data = self.data.write().await;
        data.active_character = Some(character_arc.clone());

        Ok(())
    }

    pub async fn login_active_character(&self, world: &World) -> Result<()> {
        let data = self.data.read().await;
        let character_lock = data.active_character.as_ref().unwrap();
        let character = character_lock.read().await;
        character.send_packets_before_add_to_map().await?;

        world
            .get_instance_manager()
            .get_or_create_map(&(*character), character.map)
            .await?
            .push_object(Arc::downgrade(character_lock))
            .await;

        character.send_packets_after_add_to_map(world.get_realm_database()).await?;

        Ok(())
    }
}
