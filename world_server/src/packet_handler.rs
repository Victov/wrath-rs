use super::client_manager::ClientManager;
use crate::character::character_manager::CharacterManager;
use crate::client::ClientState;
use crate::handlers::*;
use crate::prelude::*;
use crate::world::World;
use std::net::SocketAddr;
use wow_world_messages::wrath::opcodes::ClientOpcodeMessage;

pub struct PacketToHandle {
    pub client_id: SocketAddr,
    pub payload: Box<ClientOpcodeMessage>,
}

pub struct PacketHandler {}

impl PacketHandler {
    pub async fn handle_packet(
        client_manager: &mut ClientManager,
        character_manager: &mut CharacterManager,
        world: &mut World,
        packet: PacketToHandle,
    ) -> Result<()> {
        if std::env::var("PRINT_INCOMING_PACKETS")?.parse::<usize>()? == 1usize {
            info!("Incoming: {:?}", packet.payload);
        }
        {
            let client = client_manager.get_client(packet.client_id)?;
            //Most likely this won't even be reached since the client manager can't find that
            //client
            if client.data.client_state == ClientState::Disconnected {
                bail!("PacketHandler received a packet for a client that's already disconnected. Ignoring");
            }
        }

        match &*packet.payload {
            ClientOpcodeMessage::CMSG_PLAYER_LOGOUT => handle_cmsg_player_logout(client_manager, character_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_READY_FOR_ACCOUNT_DATA_TIMES => handle_csmg_ready_for_account_data_times(client_manager, &packet).await,
            ClientOpcodeMessage::CMSG_UPDATE_ACCOUNT_DATA(data) => {
                handle_csmg_update_account_data(client_manager, packet.client_id, world, data).await
            }
            ClientOpcodeMessage::CMSG_REQUEST_ACCOUNT_DATA(data) => {
                handle_cmsg_request_account_data(client_manager, packet.client_id, world, data).await
            }
            ClientOpcodeMessage::CMSG_REALM_SPLIT(data) => handle_cmsg_realm_split(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_PING(data) => handle_cmsg_ping(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_CHAR_ENUM => handle_cmsg_char_enum(client_manager, world, packet.client_id).await,
            ClientOpcodeMessage::CMSG_CHAR_CREATE(data) => handle_cmsg_char_create(client_manager, packet.client_id, world, data).await,
            ClientOpcodeMessage::CMSG_CHAR_DELETE(data) => handle_cmsg_char_delete(client_manager, packet.client_id, world, data).await,
            ClientOpcodeMessage::CMSG_PLAYER_LOGIN(data) => {
                handle_cmsg_player_login(client_manager, character_manager, world, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_STANDSTATECHANGE(data) => {
                handle_cmsg_standstate_change(client_manager, character_manager, packet.client_id, data).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_FORWARD(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_BACKWARD(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_LEFT(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_STRAFE_RIGHT(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_STRAFE(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_JUMP(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_TURN_LEFT(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_TURN_RIGHT(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_UP(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_PITCH_DOWN(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_PITCH(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_RUN_MODE(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_WALK_MODE(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_FALL_LAND(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_START_SWIM(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_SWIM(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_STOP_TURN(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_SET_FACING(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_HEARTBEAT(data) => {
                handle_movement_generic(client_manager, character_manager, packet.client_id, world, data.clone()).await
            }
            ClientOpcodeMessage::MSG_MOVE_TELEPORT_ACK(data) => {
                handle_msg_move_teleport_ack(client_manager, character_manager, packet.client_id, data).await
            }
            ClientOpcodeMessage::MSG_MOVE_WORLDPORT_ACK => {
                handle_msg_move_worldport_ack(client_manager, character_manager, packet.client_id, world).await
            }
            ClientOpcodeMessage::CMSG_WORLD_TELEPORT(data) => {
                handle_msg_world_teleport(client_manager, character_manager, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_LOGOUT_REQUEST => handle_cmsg_logout_request(client_manager, character_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_LOGOUT_CANCEL => handle_cmsg_logout_cancel(client_manager, character_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_PLAYED_TIME(data) => handle_cmsg_played_time(client_manager, character_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_QUERY_TIME => handle_cmsg_query_time(client_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_WORLD_STATE_UI_TIMER_UPDATE => handle_cmsg_world_state_ui_timer_update(client_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_TIME_SYNC_RESP(data) => {
                handle_cmsg_time_sync_resp(client_manager, character_manager, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_ZONEUPDATE(data) => handle_cmsg_zoneupdate(client_manager, character_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_AREATRIGGER(data) => handle_cmsg_areatrigger(client_manager, character_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_FORCE_MOVE_ROOT_ACK(_) => Ok(()),
            ClientOpcodeMessage::CMSG_FORCE_MOVE_UNROOT_ACK(_) => Ok(()),
            ClientOpcodeMessage::CMSG_SET_ACTIVE_MOVER(data) => handle_cmsg_set_active_mover(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_NAME_QUERY(data) => {
                handle_cmsg_name_query(client_manager, character_manager, packet.client_id, world, data).await
            }
            ClientOpcodeMessage::CMSG_TUTORIAL_FLAG(data) => {
                handle_cmsg_tutorial_flag(client_manager, character_manager, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_TUTORIAL_RESET => handle_cmsg_tutorial_reset(client_manager, character_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_SET_SELECTION(data) => {
                handle_cmsg_set_selection(client_manager, character_manager, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_JOIN_CHANNEL(data) => handle_cmsg_join_channel(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_SET_ACTIVE_VOICE_CHANNEL(_) => {
                //Voice chat is explicitly not implemented, discard message to silence warning spam
                Ok(())
            }
            ClientOpcodeMessage::CMSG_VOICE_SESSION_ENABLE(_) => {
                //Voice chat is explicitly not implemented, discard message to silence warning spam
                Ok(())
            }
            ClientOpcodeMessage::CMSG_GMTICKET_GETTICKET => handle_cmsg_gmticket_getticket(client_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_GMTICKET_CREATE(data) => handle_cmsg_gmticket_create(client_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_GMTICKET_SYSTEMSTATUS => handle_cmsg_gmticket_system_status(client_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_NEXT_CINEMATIC_CAMERA => {
                handle_csmg_next_cinematic_camera(client_manager, character_manager, packet.client_id).await
            }
            ClientOpcodeMessage::CMSG_COMPLETE_CINEMATIC => handle_csmg_complete_cinematic(client_manager, character_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_REQUEST_RAID_INFO => handle_cmsg_request_raid_info(client_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_CONTACT_LIST(data) => handle_cmsg_contact_list(client_manager, character_manager, packet.client_id, data).await,
            ClientOpcodeMessage::CMSG_CALENDAR_GET_NUM_PENDING => handle_cmsg_calendar_get_num_pending(client_manager, packet.client_id).await,
            ClientOpcodeMessage::CMSG_SET_ACTIONBAR_TOGGLES(data) => {
                handle_csmg_set_actionbar_toggles(client_manager, character_manager, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_ITEM_QUERY_SINGLE(data) => handle_cmsg_item_query_single(client_manager, packet.client_id, world, data).await,
            ClientOpcodeMessage::CMSG_ITEM_NAME_QUERY(data) => handle_cmsg_item_name_query(client_manager, packet.client_id, world, data).await,
            ClientOpcodeMessage::CMSG_SWAP_INV_ITEM(data) => {
                handle_cmsg_swap_inv_item(client_manager, character_manager, world, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_AUTOEQUIP_ITEM(data) => {
                handle_cmsg_autoequip_item(client_manager, character_manager, world, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_MESSAGECHAT(data) => {
                handle_cmsg_messagechat(client_manager, character_manager, world, packet.client_id, data).await
            }
            ClientOpcodeMessage::CMSG_SET_ACTION_BUTTON(data) => {
                handle_cmsg_set_action_button(client_manager, character_manager, packet.client_id, data).await
            }
            _ => bail!("Unhandled opcode"),
        }
    }
}
