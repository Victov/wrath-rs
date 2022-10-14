use super::client_manager::ClientManager;
use crate::client::ClientState;
use crate::handlers::*;
use crate::prelude::*;
use crate::world::World;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use wow_world_messages::wrath::opcodes::ClientOpcodeMessage;

pub struct PacketToHandle {
    pub client_id: u64,
    pub payload: Box<ClientOpcodeMessage>,
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

    pub async fn handle_queue(&self, client_manager: Arc<ClientManager>, world: Arc<World>) -> Result<()> {
        for packet in self.receive_channel.try_iter() {
            self.handle_packet(&client_manager, &world, &packet).await.unwrap_or_else(|e| {
                warn!("Error while handling packet {:?}: {}", packet.payload, e);
            });
        }

        Ok(())
    }

    async fn handle_packet(&self, client_manager: &ClientManager, world: &World, packet: &PacketToHandle) -> Result<()> {
        if std::env::var("PRINT_INCOMING_PACKETS")?.parse::<usize>()? == 1usize {
            info!("Incoming: {:?}", packet.payload);
        }
        {
            let client = client_manager.get_client(packet.client_id).await?;
            //Most likely this won't even be reached since the client manager can't find that
            //client
            if client.data.read().await.client_state == ClientState::Disconnected {
                bail!("PacketHandler received a packet for a client that's already disconnected. Ignoring");
            }
        }

        match &*packet.payload {
            ClientOpcodeMessage::CMSG_READY_FOR_ACCOUNT_DATA_TIMES(_) => handle_csmg_ready_for_account_data_times(client_manager, packet).await,
            ClientOpcodeMessage::CMSG_UPDATE_ACCOUNT_DATA(data) => {
                handle_csmg_update_account_data(client_manager, packet.client_id, world, data).await
            }
            ClientOpcodeMessage::CMSG_REQUEST_ACCOUNT_DATA(data) => {
                handle_cmsg_request_account_data(client_manager, packet.client_id, world, data).await
            }
            ClientOpcodeMessage::CMSG_REALM_SPLIT(data) => handle_cmsg_realm_split(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_PING(data) => handle_cmsg_ping(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_CHAR_ENUM(_) => handle_cmsg_char_enum(client_manager, world, packet.client_id).await,
            ClientOpcodeMessage::CMSG_CHAR_CREATE(data) => handle_cmsg_char_create(client_manager, packet.client_id, world, data).await,
            ClientOpcodeMessage::CMSG_PLAYER_LOGIN(data) => handle_cmsg_player_login(client_manager, world, packet.client_id, data).await,
            ClientOpcodeMessage::MSG_MOVE_START_FORWARD(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_START_BACKWARD(data) => {
                handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_LEFT(data) => {
                handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_RIGHT(data) => {
                handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_STRAFE(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_JUMP(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_START_TURN_LEFT(data) => {
                handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_TURN_RIGHT(data) => {
                handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_UP(data) => {
                handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_DOWN(data) => {
                handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_PITCH(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_SET_RUN_MODE(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_SET_WALK_MODE(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_FALL_LAND(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_START_SWIM(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_STOP_SWIM(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_SET_FACING(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_HEARTBEAT(data) => handle_movement_generic(client_manager, packet.client_id, world, data.clone()).await,
            ClientOpcodeMessage::MSG_MOVE_TELEPORT_ACK(data) => handle_msg_move_teleport_ack(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::MSG_MOVE_WORLDPORT_ACK(data) => handle_msg_move_worldport_ack(client_manager, packet.client_id, world, data).await,
            ClientOpcodeMessage::CMSG_LOGOUT_REQUEST(data) => handle_cmsg_logout_request(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_LOGOUT_CANCEL(data) => handle_cmsg_logout_cancel(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_PLAYED_TIME(data) => handle_cmsg_played_time(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_QUERY_TIME(data) => handle_cmsg_query_time(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_WORLD_STATE_UI_TIMER_UPDATE(data) => {
                handle_cmsg_world_state_ui_timer_update(client_manager, packet.client_id, data).await
            }
            _ => bail!("Unhandled opcode"),
        }
    }
}

/*
            Opcodes::CMSG_TUTORIAL_FLAG => handle_cmsg_tutorial_flag(client_manager, packet).await,
            Opcodes::CMSG_NAME_QUERY => handle_cmsg_name_query(client_manager, world, packet).await,
            Opcodes::CMSG_ZONEUPDATE => handle_cmsg_zoneupdate(client_manager, packet).await,
            Opcodes::CMSG_TIME_SYNC_RESP => handle_cmsg_time_sync_resp(client_manager, packet).await,
            Opcodes::CMSG_FORCE_MOVE_ROOT_ACK => Ok(()),   //Don't do anything, disable warning spam
            Opcodes::CMSG_FORCE_MOVE_UNROOT_ACK => Ok(()), //Don't do anything, disable warning spam
            Opcodes::CMSG_AREATRIGGER => handle_cmsg_areatrigger(client_manager, packet).await,
            Opcodes::CMSG_ITEM_QUERY_SINGLE => handle_cmsg_item_query_single(client_manager, packet, world).await,

*/
