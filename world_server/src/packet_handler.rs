use super::client_manager::ClientManager;
use crate::handlers::*;
use crate::opcodes::Opcodes;
use crate::packet::ClientPacketHeader;
use crate::world::World;
use anyhow::Result;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub struct PacketToHandle {
    pub header: ClientPacketHeader,
    pub client_id: u64,
    pub payload: Vec<u8>,
}

pub struct PacketHandler {
    receive_channel: Receiver<PacketToHandle>,
    world: Arc<World>,
}

impl PacketHandler {
    pub fn new(packet_receiver_channel: Receiver<PacketToHandle>, world: Arc<World>) -> Self {
        Self {
            receive_channel: packet_receiver_channel,
            world,
        }
    }

    pub async fn handle_queue(&self, client_manager: &Arc<ClientManager>) -> Result<()> {
        while let Ok(packet) = self.receive_channel.try_recv() {
            self.handle_packet(client_manager, &packet).await?;
        }

        Ok(())
    }

    async fn handle_packet(&self, client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
        if std::env::var("PRINT_INCOMING_PACKETS")?.parse::<usize>()? == 1usize {
            println!("Incoming: {:?}", packet.header.get_cmd());
        }
        match packet.header.get_cmd()? {
            Opcodes::CMSG_AUTH_SESSION => handle_cmsg_auth_session(client_manager, packet).await,
            Opcodes::CMSG_READY_FOR_ACCOUNT_DATA_TIMES => handle_csmg_ready_for_account_data_times(client_manager, packet).await,
            Opcodes::CMSG_CHAR_ENUM => handle_cmsg_char_enum(client_manager, packet).await,
            Opcodes::CMSG_REALM_SPLIT => handle_cmsg_realm_split(client_manager, packet).await,
            Opcodes::CMSG_CHAR_CREATE => handle_cmsg_char_create(client_manager, packet).await,
            Opcodes::CMSG_PING => handle_cmsg_ping(client_manager, packet).await,
            Opcodes::CMSG_UPDATE_ACCOUNT_DATA => handle_csmg_update_account_data(client_manager, packet).await,
            Opcodes::CMSG_PLAYER_LOGIN => handle_cmsg_player_login(client_manager, packet).await,
            op => Err(anyhow::anyhow!("Unhandled opcode {:?}", op)),
        }
    }
}
