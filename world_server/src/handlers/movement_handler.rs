use crate::character::Character;
use crate::client_manager::ClientManager;
use crate::data::{AreaTriggerPurpose, PositionAndOrientation, WorldZoneLocation};
use crate::packet::ServerMessageExt;
use crate::prelude::*;
use crate::world::prelude::GameObject;
use crate::world::World;
use async_std::sync::RwLockUpgradableReadGuard;
use std::sync::Arc;
use wow_world_messages::wrath::{
    Area, ClientMessage, MSG_MOVE_TELEPORT_ACK_Client, MSG_MOVE_TELEPORT_ACK_Server, Map, MovementInfo, ServerMessage, UnitStandState, Vector3d,
    CMSG_AREATRIGGER, MSG_MOVE_FALL_LAND, MSG_MOVE_HEARTBEAT, MSG_MOVE_JUMP, MSG_MOVE_SET_FACING, MSG_MOVE_SET_RUN_MODE, MSG_MOVE_SET_WALK_MODE,
    MSG_MOVE_START_BACKWARD, MSG_MOVE_START_FORWARD, MSG_MOVE_START_PITCH_DOWN, MSG_MOVE_START_PITCH_UP, MSG_MOVE_START_STRAFE_LEFT,
    MSG_MOVE_START_STRAFE_RIGHT, MSG_MOVE_START_SWIM, MSG_MOVE_START_TURN_LEFT, MSG_MOVE_START_TURN_RIGHT, MSG_MOVE_STOP, MSG_MOVE_STOP_PITCH,
    MSG_MOVE_STOP_STRAFE, MSG_MOVE_STOP_SWIM, MSG_MOVE_STOP_TURN, MSG_MOVE_WORLDPORT_ACK, SMSG_FORCE_MOVE_ROOT, SMSG_FORCE_MOVE_UNROOT,
    SMSG_NEW_WORLD, SMSG_STANDSTATE_UPDATE, SMSG_TRANSFER_PENDING,
};

pub trait MovementMessage: Sync + ServerMessage + ClientMessage {
    fn get_guid(&self) -> Guid;
    fn get_movement_info(&self) -> MovementInfo;
}

macro_rules! define_movement_packet {
    ($packet_type:ty) => {
        impl MovementMessage for $packet_type {
            fn get_guid(&self) -> Guid {
                self.guid
            }

            fn get_movement_info(&self) -> MovementInfo {
                self.info.clone()
            }
        }
    };
}

define_movement_packet!(MSG_MOVE_START_FORWARD);
define_movement_packet!(MSG_MOVE_START_BACKWARD);
define_movement_packet!(MSG_MOVE_STOP);
define_movement_packet!(MSG_MOVE_STOP_TURN);
define_movement_packet!(MSG_MOVE_START_STRAFE_LEFT);
define_movement_packet!(MSG_MOVE_START_STRAFE_RIGHT);
define_movement_packet!(MSG_MOVE_STOP_STRAFE);
define_movement_packet!(MSG_MOVE_JUMP);
define_movement_packet!(MSG_MOVE_START_TURN_LEFT);
define_movement_packet!(MSG_MOVE_START_TURN_RIGHT);
define_movement_packet!(MSG_MOVE_START_PITCH_UP);
define_movement_packet!(MSG_MOVE_START_PITCH_DOWN);
define_movement_packet!(MSG_MOVE_STOP_PITCH);
define_movement_packet!(MSG_MOVE_SET_RUN_MODE);
define_movement_packet!(MSG_MOVE_SET_WALK_MODE);
define_movement_packet!(MSG_MOVE_FALL_LAND);
define_movement_packet!(MSG_MOVE_START_SWIM);
define_movement_packet!(MSG_MOVE_STOP_SWIM);
define_movement_packet!(MSG_MOVE_SET_FACING);
define_movement_packet!(MSG_MOVE_HEARTBEAT);

pub async fn handle_movement_generic<T: MovementMessage>(client_manager: &ClientManager, client_id: u64, world: &World, packet: T) -> Result<()> {
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

    {
        let mut character = character_lock.write().await;
        character.process_movement(movement_info);
    }

    let character = character_lock.read().await;
    packet.astd_send_to_all_in_range(&*character, false, world).await
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

pub async fn send_msg_move_teleport_ack(character: &Character, destination: &PositionAndOrientation) -> Result<()> {
    let mut movement_info = character.get_movement_info().clone();
    movement_info.position = destination.position;
    movement_info.orientation = destination.orientation;

    MSG_MOVE_TELEPORT_ACK_Server {
        guid: character.get_guid(),
        movement_counter: 0, //TODO: Value should increment with every teleport?
        info: movement_info,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn send_smsg_transfer_pending(character: &Character, map: Map) -> Result<()> {
    SMSG_TRANSFER_PENDING { map, has_transport: None }.astd_send_to_character(character).await
}

pub async fn send_smsg_new_world(character: &Character, map: Map, position: PositionAndOrientation) -> Result<()> {
    SMSG_NEW_WORLD {
        map,
        position: position.position,
        orientation: position.orientation,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn handle_msg_move_teleport_ack(client_manager: &ClientManager, client_id: u64, _packet: &MSG_MOVE_TELEPORT_ACK_Client) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;
    let character = character_lock.upgradable_read().await;

    if let TeleportationState::Executing(TeleportationDistance::Near(destination)) = character.teleportation_state.clone() {
        let mut character = RwLockUpgradableReadGuard::upgrade(character).await;
        character.set_position(&destination);
        character.teleportation_state = TeleportationState::None;
    }

    Ok(())
}

pub async fn handle_msg_move_worldport_ack(
    client_manager: &ClientManager,
    client_id: u64,
    world: &World,
    _packet: &MSG_MOVE_WORLDPORT_ACK,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
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
    SMSG_STANDSTATE_UPDATE { state: stand_state }.astd_send_to_character(character).await
}

pub async fn send_smsg_force_move_root(character: &Character) -> Result<()> {
    SMSG_FORCE_MOVE_ROOT {
        guid: character.get_guid(),
        counter: 0,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn send_smsg_force_move_unroot(character: &Character) -> Result<()> {
    SMSG_FORCE_MOVE_UNROOT {
        guid: character.get_guid(),
        counter: 0,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn handle_cmsg_areatrigger(client_manager: &ClientManager, client_id: u64, packet: &CMSG_AREATRIGGER) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let area_trigger_id = packet.trigger_id;

    let trigger_data = client_manager
        .data_storage
        .get_area_trigger(area_trigger_id)
        .ok_or_else(|| anyhow!("Character entered area trigger that isn't known to the server"))?;

    if let AreaTriggerPurpose::Teleport(teleport_data) = &trigger_data.purpose {
        let mut character = character_lock.write().await;
        let destination = WorldZoneLocation {
            position: Vector3d {
                x: teleport_data.target_position_x,
                y: teleport_data.target_position_y,
                z: teleport_data.target_position_z,
            },
            orientation: teleport_data.target_orientation,
            map: (teleport_data.target_map as u32).try_into()?,
            area: Area::NorthshireValley, //TODO
        };
        character.teleport_to(TeleportationDistance::Far(destination))
    } else if let AreaTriggerPurpose::RestedArea = &trigger_data.purpose {
        let mut character = character_lock.write().await;
        character.handle_enter_inn()?;
    }
    Ok(())
}
