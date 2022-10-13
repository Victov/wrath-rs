use crate::client_manager::ClientManager;
use crate::data::{PositionAndOrientation, WorldZoneLocation};
use crate::packet::ServerMessageExt;
use crate::prelude::*;
use crate::world::World;
use wow_world_messages::wrath::{
    MSG_MOVE_FALL_LAND_Client, MSG_MOVE_FALL_LAND_Server, MSG_MOVE_HEARTBEAT_Client, MSG_MOVE_HEARTBEAT_Server, MSG_MOVE_JUMP_Client,
    MSG_MOVE_JUMP_Server, MSG_MOVE_SET_FACING_Client, MSG_MOVE_SET_FACING_Server, MSG_MOVE_SET_RUN_MODE_Client, MSG_MOVE_SET_RUN_MODE_Server,
    MSG_MOVE_SET_WALK_MODE_Client, MSG_MOVE_SET_WALK_MODE_Server, MSG_MOVE_START_BACKWARD_Client, MSG_MOVE_START_BACKWARD_Server,
    MSG_MOVE_START_FORWARD_Client, MSG_MOVE_START_FORWARD_Server, MSG_MOVE_START_PITCH_DOWN_Client, MSG_MOVE_START_PITCH_DOWN_Server,
    MSG_MOVE_START_PITCH_UP_Client, MSG_MOVE_START_PITCH_UP_Server, MSG_MOVE_START_STRAFE_LEFT_Client, MSG_MOVE_START_STRAFE_LEFT_Server,
    MSG_MOVE_START_STRAFE_RIGHT_Client, MSG_MOVE_START_STRAFE_RIGHT_Server, MSG_MOVE_START_SWIM_Client, MSG_MOVE_START_SWIM_Server,
    MSG_MOVE_START_TURN_LEFT_Client, MSG_MOVE_START_TURN_LEFT_Server, MSG_MOVE_START_TURN_RIGHT_Client, MSG_MOVE_START_TURN_RIGHT_Server,
    MSG_MOVE_STOP_Client, MSG_MOVE_STOP_PITCH_Client, MSG_MOVE_STOP_PITCH_Server, MSG_MOVE_STOP_STRAFE_Client, MSG_MOVE_STOP_STRAFE_Server,
    MSG_MOVE_STOP_SWIM_Client, MSG_MOVE_STOP_SWIM_Server, MSG_MOVE_STOP_Server, MovementInfo, ServerMessage,
};

pub trait ClientMovementMessage {
    type OutgoingType: ServerMessage + Sync;
    fn get_guid(&self) -> Guid;
    fn get_movement_info(&self) -> MovementInfo;
    fn into_outgoing_type(self) -> Self::OutgoingType;
}

macro_rules! define_movement_packet_pair {
    ($client_packet_type:ty, $server_packet_type:ty) => {
        impl ClientMovementMessage for $client_packet_type {
            type OutgoingType = $server_packet_type;
            fn get_guid(&self) -> Guid {
                self.guid
            }

            fn get_movement_info(&self) -> MovementInfo {
                self.info.clone()
            }

            fn into_outgoing_type(self) -> Self::OutgoingType {
                Self::OutgoingType {
                    guid: self.guid,
                    info: self.info,
                }
            }
        }
    };
}

define_movement_packet_pair!(MSG_MOVE_START_FORWARD_Client, MSG_MOVE_START_FORWARD_Server);
define_movement_packet_pair!(MSG_MOVE_START_BACKWARD_Client, MSG_MOVE_START_BACKWARD_Server);
define_movement_packet_pair!(MSG_MOVE_STOP_Client, MSG_MOVE_STOP_Server);
define_movement_packet_pair!(MSG_MOVE_START_STRAFE_LEFT_Client, MSG_MOVE_START_STRAFE_LEFT_Server);
define_movement_packet_pair!(MSG_MOVE_START_STRAFE_RIGHT_Client, MSG_MOVE_START_STRAFE_RIGHT_Server);
define_movement_packet_pair!(MSG_MOVE_STOP_STRAFE_Client, MSG_MOVE_STOP_STRAFE_Server);
define_movement_packet_pair!(MSG_MOVE_JUMP_Client, MSG_MOVE_JUMP_Server);
define_movement_packet_pair!(MSG_MOVE_START_TURN_LEFT_Client, MSG_MOVE_START_TURN_LEFT_Server);
define_movement_packet_pair!(MSG_MOVE_START_TURN_RIGHT_Client, MSG_MOVE_START_TURN_RIGHT_Server);
define_movement_packet_pair!(MSG_MOVE_START_PITCH_UP_Client, MSG_MOVE_START_PITCH_UP_Server);
define_movement_packet_pair!(MSG_MOVE_START_PITCH_DOWN_Client, MSG_MOVE_START_PITCH_DOWN_Server);
define_movement_packet_pair!(MSG_MOVE_STOP_PITCH_Client, MSG_MOVE_STOP_PITCH_Server);
define_movement_packet_pair!(MSG_MOVE_SET_RUN_MODE_Client, MSG_MOVE_SET_RUN_MODE_Server);
define_movement_packet_pair!(MSG_MOVE_SET_WALK_MODE_Client, MSG_MOVE_SET_WALK_MODE_Server);
define_movement_packet_pair!(MSG_MOVE_FALL_LAND_Client, MSG_MOVE_FALL_LAND_Server);
define_movement_packet_pair!(MSG_MOVE_START_SWIM_Client, MSG_MOVE_START_SWIM_Server);
define_movement_packet_pair!(MSG_MOVE_STOP_SWIM_Client, MSG_MOVE_STOP_SWIM_Server);
define_movement_packet_pair!(MSG_MOVE_SET_FACING_Client, MSG_MOVE_SET_FACING_Server);
define_movement_packet_pair!(MSG_MOVE_HEARTBEAT_Client, MSG_MOVE_HEARTBEAT_Server);

pub async fn handle_movement_generic<T: ClientMovementMessage>(
    client_manager: &ClientManager,
    client_id: u64,
    world: &World,
    packet: T,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;
    {
        let character = character_lock.read().await;
        if character.teleportation_state != TeleportationState::None {
            //Not an error, but we do simply want to ignore these packet
            return Ok(());
        }
    }

    let _guid = packet.get_guid();
    let movement_info = packet.get_movement_info();
    let server_outgoing_packet = packet.into_outgoing_type();

    {
        let mut character = character_lock.write().await;
        character.process_movement(movement_info);
    }

    let character = character_lock.read().await;
    server_outgoing_packet.astd_send_to_all_in_range(&*character, false, world).await
}

#[derive(PartialEq, Debug, Clone)]
pub enum TeleportationState {
    None,
    Queued(TeleportationDistance),
    Executing(TeleportationDistance),
}

#[derive(PartialEq, Debug, Clone)]
pub enum TeleportationDistance {
    Near(PositionAndOrientation),
    Far(WorldZoneLocation),
}

/*
pub async fn send_msg_move_teleport_ack(character: &Character, destination: &PositionAndOrientation) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::MSG_MOVE_TELEPORT_ACK, 64);

    let mut movement_info = character.movement_info.clone();
    movement_info.position = destination.clone();

    writer.write_guid_compressed(&character.guid)?;
    writer.write_u32::<LittleEndian>(0)?; //This value is supposed to increment with every teleport?
    writer.write_movement_info(&movement_info)?;

    send_packet_to_character(character, &header, &writer).await
}

pub async fn send_smsg_transfer_pending(character: &Character, map_id: u32) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_TRANSFER_PENDING, 12);
    writer.write_u32::<LittleEndian>(map_id)?;
    send_packet_to_character(character, &header, &writer).await
}

pub async fn send_smsg_new_world(character: &Character, map_id: u32, position: &PositionAndOrientation) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_NEW_WORLD, 20);
    writer.write_u32::<LittleEndian>(map_id)?;
    writer.write_position_and_orientation(position)?;
    send_packet_to_character(character, &header, &writer).await
}

pub async fn handle_msg_move_teleport_ack(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let character_lock = client.get_active_character().await?;

    let character = character_lock.upgradable_read().await;

    if let TeleportationState::Executing(TeleportationDistance::Near(destination)) = character.teleportation_state.clone() {
        let mut reader = std::io::Cursor::new(&packet.payload);

        //TODO: check validity of these returned things.
        let _mover_guid = reader.read_guid_compressed()?;
        let _counter = reader.read_u32::<LittleEndian>()?;
        let _time = reader.read_u32::<LittleEndian>()?;

        let mut character = RwLockUpgradableReadGuard::upgrade(character).await;
        character.set_position(&destination);
        character.teleportation_state = TeleportationState::None;
    }

    Ok(())
}

pub async fn handle_msg_move_worldport_ack(client_manager: &ClientManager, world: &World, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let character_lock = client.get_active_character().await?;

    let character = character_lock.upgradable_read().await;

    if let TeleportationState::Executing(TeleportationDistance::Far(destination)) = character.teleportation_state.clone() {
        let mut character = RwLockUpgradableReadGuard::upgrade(character).await;

        let map = world.get_instance_manager().get_or_create_map(&(*character), destination.map).await?;

        character.map = destination.map;
        character.set_position(&destination.into());
        character.reset_time_sync();
        character.send_packets_before_add_to_map().await?;
        map.push_object(Arc::downgrade(&character_lock)).await;
        character.send_packets_after_add_to_map(world.get_realm_database()).await?;

        character.teleportation_state = TeleportationState::None;
    }

    Ok(())
}

pub async fn send_smsg_stand_state_update(character: &Character, stand_state: UnitStandState) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_STANDSTATE_UPDATE, 1);
    writer.write_u8(stand_state as u8)?;
    send_packet_to_character(character, &header, &writer).await
}

pub async fn send_smsg_force_move_root(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_FORCE_MOVE_ROOT, 4);
    writer.write_guid_compressed(character.get_guid())?;
    writer.write_u32::<LittleEndian>(0)?;
    send_packet_to_character(character, &header, &writer).await
}

pub async fn send_smsg_force_move_unroot(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_FORCE_MOVE_UNROOT, 4);
    writer.write_guid_compressed(character.get_guid())?;
    writer.write_u32::<LittleEndian>(0)?;
    send_packet_to_character(character, &header, &writer).await
}

pub async fn handle_cmsg_areatrigger(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let character_lock = client.get_active_character().await?;

    let area_trigger_id = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_u32::<LittleEndian>()?
    };

    let trigger_data = client_manager
        .data_storage
        .get_area_trigger(area_trigger_id)
        .ok_or_else(|| anyhow!("Character entered area trigger that isn't known to the server"))?;

    if let AreaTriggerPurpose::Teleport(teleport_data) = &trigger_data.purpose {
        let mut character = character_lock.write().await;
        let destination = WorldZoneLocation {
            x: teleport_data.target_position_x,
            y: teleport_data.target_position_y,
            z: teleport_data.target_position_z,
            o: teleport_data.target_orientation,
            map: teleport_data.target_map as u32,
            zone: 0, //todo?
        };
        character.teleport_to(TeleportationDistance::Far(destination))
    } else if let AreaTriggerPurpose::RestedArea = &trigger_data.purpose {
        let mut character = character_lock.write().await;
        character.handle_enter_inn()?;
    }
    Ok(())
}
*/
