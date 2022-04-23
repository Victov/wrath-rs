use super::client_manager::ClientManager;
use crate::client::ClientState;
use crate::handlers::*;
use crate::opcodes::Opcodes;
use crate::packet::ClientPacketHeader;
use crate::prelude::*;
use crate::world::World;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub struct PacketToHandle {
    pub header: ClientPacketHeader,
    pub client_id: u64,
    pub payload: Vec<u8>,
}

pub struct PacketHandler {
    receive_channel: Receiver<PacketToHandle>,
}

impl PacketHandler {
    pub fn new(packet_receiver_channel: Receiver<PacketToHandle>, _world: Arc<World>) -> Self {
        Self {
            receive_channel: packet_receiver_channel,
        }
    }

    pub async fn handle_queue(&self, client_manager: &Arc<ClientManager>) -> Result<()> {
        for packet in self.receive_channel.try_iter() {
            self.handle_packet(client_manager, &packet).await.unwrap_or_else(|e| {
                warn!("Error while handling packet: {}", e);
            });
        }

        Ok(())
    }

    async fn handle_packet(&self, client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
        if std::env::var("PRINT_INCOMING_PACKETS")?.parse::<usize>()? == 1usize {
            info!("Incoming: {:?}", packet.header.get_cmd());
        }
        {
            let client_lock = client_manager.get_client(packet.client_id).await?;
            let client = client_lock.read().await;
            //Most likely this won't even be reached since the client manager can't find that
            //client
            if client.client_state == ClientState::Disconnected {
                bail!("PacketHandler received a packet for a client that's already disconnected. Ignoring");
            }
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
            Opcodes::CMSG_REQUEST_ACCOUNT_DATA => handle_cmsg_request_account_data(client_manager, packet).await,
            Opcodes::CMSG_PLAYED_TIME => handle_cmsg_played_time(client_manager, packet).await,
            Opcodes::CMSG_QUERY_TIME => handle_cmsg_query_time(client_manager, packet).await,
            Opcodes::CMSG_WORLD_STATE_UI_TIMER_UPDATE => handle_cmsg_world_state_ui_timer_update(client_manager, packet).await,
            Opcodes::CMSG_TUTORIAL_FLAG => handle_cmsg_tutorial_flag(client_manager, packet).await,
            Opcodes::CMSG_NAME_QUERY => handle_cmsg_name_query(client_manager, packet).await,
            Opcodes::CMSG_SET_ACTIONBAR_TOGGLES => handle_cmsg_set_actionbar_toggles(client_manager, packet).await,
            Opcodes::CMSG_ZONEUPDATE => handle_cmsg_zoneupdate(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_FORWARD => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_BACKWARD => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_STOP => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_STRAFE_LEFT => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_STRAFE_RIGHT => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_STOP_STRAFE => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_JUMP => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_TURN_LEFT => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_TURN_RIGHT => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_STOP_TURN => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_PITCH_UP => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_PITCH_DOWN => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_STOP_PITCH => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_SET_RUN_MODE => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_SET_WALK_MODE => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_FALL_LAND => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_START_SWIM => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_STOP_SWIM => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_SET_FACING => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_SET_PITCH => handle_movement_generic(client_manager, packet).await,
            Opcodes::MSG_MOVE_HEARTBEAT => handle_movement_generic(client_manager, packet).await,
            Opcodes::CMSG_TIME_SYNC_RESP => handle_cmsg_time_sync_resp(client_manager, packet).await,
            Opcodes::MSG_MOVE_TELEPORT_ACK => handle_msg_move_teleport_ack(client_manager, packet).await,

            op => Err(anyhow!("Unhandled opcode {:?}", op)),
        }
    }
}
