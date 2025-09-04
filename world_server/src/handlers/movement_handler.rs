use crate::character::character_manager::CharacterManager;
use crate::character::Character;
use crate::client_manager::ClientManager;
use crate::connection::events::{IntoServerEvent, ServerEvent};
use crate::data::{AreaTriggerPurpose, PositionAndOrientation, WorldZoneLocation};
use crate::prelude::*;
use crate::world::prelude::GameObject;
use crate::world::World;
use std::net::SocketAddr;
use wow_world_messages::wrath::{
    Area, ClientMessage, MSG_MOVE_TELEPORT_ACK_Client, MSG_MOVE_TELEPORT_ACK_Server, Map, MovementInfo, ServerMessage, UnitStandState, Vector3d,
    CMSG_AREATRIGGER, CMSG_SET_ACTIVE_MOVER, CMSG_WORLD_TELEPORT, MSG_MOVE_FALL_LAND, MSG_MOVE_HEARTBEAT, MSG_MOVE_JUMP, MSG_MOVE_SET_FACING,
    MSG_MOVE_SET_RUN_MODE, MSG_MOVE_SET_WALK_MODE, MSG_MOVE_START_BACKWARD, MSG_MOVE_START_FORWARD, MSG_MOVE_START_PITCH_DOWN,
    MSG_MOVE_START_PITCH_UP, MSG_MOVE_START_STRAFE_LEFT, MSG_MOVE_START_STRAFE_RIGHT, MSG_MOVE_START_SWIM, MSG_MOVE_START_TURN_LEFT,
    MSG_MOVE_START_TURN_RIGHT, MSG_MOVE_STOP, MSG_MOVE_STOP_PITCH, MSG_MOVE_STOP_STRAFE, MSG_MOVE_STOP_SWIM, MSG_MOVE_STOP_TURN,
    SMSG_FORCE_MOVE_ROOT, SMSG_FORCE_MOVE_UNROOT, SMSG_NEW_WORLD, SMSG_STANDSTATE_UPDATE, SMSG_TRANSFER_PENDING,
};

pub trait MovementMessage: Sync + ServerMessage + ClientMessage + IntoServerEvent {
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

pub async fn handle_movement_generic<T: MovementMessage>(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    world: &World,
    packet: T,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    {
        let character = character_manager.get_character_mut(guid)?;
        if character.teleportation_state != TeleportationState::None {
            //Not an error, but we do simply want to ignore these packet
            return Ok(());
        }

        let _guid = packet.get_guid();
        let movement_info = packet.get_movement_info();

        character.process_movement(movement_info);
    }

    let character = character_manager.get_character(guid)?;
    packet
        .into_server_event()
        .send_to_all_in_range(character, character_manager, false, world)
        .await
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

    ServerEvent::MoveTeleportAck(MSG_MOVE_TELEPORT_ACK_Server {
        guid: character.get_guid(),
        movement_counter: 0, //TODO: Value should increment with every teleport?
        info: movement_info,
    })
    .send_to_character(character)
    .await
}

pub async fn send_smsg_transfer_pending(character: &Character, map: Map) -> Result<()> {
    ServerEvent::TransferPending(SMSG_TRANSFER_PENDING { map, has_transport: None })
        .send_to_character(character)
        .await
}

pub async fn send_smsg_new_world(character: &Character, map: Map, position: PositionAndOrientation) -> Result<()> {
    ServerEvent::NewWorld(SMSG_NEW_WORLD {
        map,
        position: position.position,
        orientation: position.orientation,
    })
    .send_to_character(character)
    .await
}

pub async fn handle_msg_move_teleport_ack(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    _packet: &MSG_MOVE_TELEPORT_ACK_Client,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character_mut(guid)?;

    if let TeleportationState::Executing(TeleportationDistance::Near(destination)) = character.teleportation_state.clone() {
        character.set_position(&destination);
        character.teleportation_state = TeleportationState::None;
    }

    Ok(())
}

pub async fn handle_msg_move_worldport_ack(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    world: &mut World,
) -> Result<()> {
    let teleportation_state = {
        let client = client_manager.get_authenticated_client(client_id)?;
        let guid = client.get_active_character();
        let character = character_manager.get_character_mut(guid)?;
        character.teleportation_state.clone()
    };

    if let TeleportationState::Executing(TeleportationDistance::Far(destination)) = teleportation_state {
        let map = destination.map;
        {
            let guid = client_manager.get_character_from_client(client_id).await?;
            let character = character_manager.get_character_mut(guid)?;
            let _ = world.get_instance_manager_mut().get_or_create_map(character, map).await?;
            character.map = map;
            character.set_position(&destination.into());
            character.reset_time_sync();
        }

        let client = client_manager.get_authenticated_client(client_id)?;
        let guid = client.get_active_character();
        let character = character_manager.get_character(guid)?;
        character.send_packets_before_add_to_map().await?;

        let map = world.get_instance_manager_mut().get_or_create_map(character, map).await?;
        map.push_character(character);
        character.send_packets_after_add_to_map(world.get_realm_database()).await?;
        {
            let guid = client_manager.get_character_from_client(client_id).await?;
            let character = character_manager.get_character_mut(guid)?;
            character.teleportation_state = TeleportationState::None;
        }
    }

    Ok(())
}

pub async fn handle_msg_world_teleport(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_WORLD_TELEPORT,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character_mut(guid)?;

    info!("Teleporting character {} to {} ({:?})", character.name, packet.map, packet.position);

    let destination = WorldZoneLocation {
        position: packet.position,
        orientation: packet.orientation,
        map: packet.map,
        area: Area::NorthshireValley, // TODO: Work out area from position + map.
    };
    character.teleport_to(TeleportationDistance::Far(destination));

    Ok(())
}

pub async fn send_smsg_stand_state_update(character: &Character, stand_state: UnitStandState) -> Result<()> {
    ServerEvent::StandStateUpdate(SMSG_STANDSTATE_UPDATE { state: stand_state })
        .send_to_character(character)
        .await
}

pub async fn send_smsg_force_move_root(character: &Character) -> Result<()> {
    ServerEvent::ForceMoveRoot(SMSG_FORCE_MOVE_ROOT {
        guid: character.get_guid(),
        counter: 0,
    })
    .send_to_character(character)
    .await
}

pub async fn send_smsg_force_move_unroot(character: &Character) -> Result<()> {
    ServerEvent::ForceMoveUnroot(SMSG_FORCE_MOVE_UNROOT {
        guid: character.get_guid(),
        counter: 0,
    })
    .send_to_character(character)
    .await
}

pub async fn handle_cmsg_set_active_mover(client_manager: &ClientManager, client_id: SocketAddr, packet: &CMSG_SET_ACTIVE_MOVER) -> Result<()> {
    //Many other emulators only do some verification upon receiving this packet.
    //Maybe it doesn't serve any other purpose but to have the server check it's content
    //but I have a feeling the actual server does more with this...

    let client = client_manager.get_authenticated_client(client_id)?;
    let character_guid = client.get_active_character();

    let mover_guid = packet.guid;
    //TODO: check against the character->mover, but since moving anything other than the character
    //itself (e.g. mind control) isn't implemented yet, we expect the character guid.
    //This warning will be false negative once stuff like mindcontrol is implemented, and must be
    //fixed then.
    if character_guid != mover_guid {
        warn!("Unexpected mover guid sent by the client");
    }
    Ok(())
}

pub async fn handle_cmsg_areatrigger(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_AREATRIGGER,
) -> Result<()> {
    let area_trigger_id = packet.trigger_id;

    let trigger_data = client_manager
        .data_storage
        .get_area_trigger(area_trigger_id as i32)
        .ok_or_else(|| anyhow!("Character entered area trigger that isn't known to the server"))?;

    if let AreaTriggerPurpose::Teleport(teleport_data) = &trigger_data.purpose {
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

        let client = client_manager.get_authenticated_client(client_id)?;
        let character = character_manager.get_character_mut(client.get_active_character())?;
        character.teleport_to(TeleportationDistance::Far(destination))
    } else if let AreaTriggerPurpose::RestedArea = &trigger_data.purpose {
        let client = client_manager.get_authenticated_client(client_id)?;
        let character = character_manager.get_character_mut(client.get_active_character())?;
        character.handle_enter_inn()?;
    }
    Ok(())
}
