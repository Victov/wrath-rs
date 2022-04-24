use crate::character::Character;
use crate::client_manager::ClientManager;
use crate::data::{guid::*, PositionAndOrientation, ReadMovementInfo, WorldZoneLocation, WriteMovementInfo, WritePositionAndOrientation};
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use async_std::sync::RwLockUpgradableReadGuard;
use podio::{LittleEndian, ReadPodExt, WritePodExt};
use std::sync::Arc;

pub async fn handle_movement_generic(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;
    if !client.is_authenticated() {
        bail!("Trying to request playtime for character that isn't authenticated");
    }
    let character_lock = client
        .active_character
        .as_ref()
        .ok_or(anyhow!("Trying to obtain played time, but no character is active for this client"))?
        .clone();

    let mut reader = std::io::Cursor::new(&packet.payload);
    let guid = reader.read_guid_compressed()?;
    let movement_info = reader.read_movement_info()?;

    {
        let mut character = character_lock.write().await;
        character.process_movement(&movement_info);
    }

    let (header, mut writer) = create_packet(packet.header.get_cmd()?, 8);
    writer.write_guid_compressed(&guid)?;
    writer.write_movement_info(&movement_info)?;

    let character = character_lock.read().await;
    send_packet_to_all_in_range(&character, false, &client_manager.world, &header, &writer).await?;

    Ok(())
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

pub async fn handle_msg_move_teleport_ack(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;
    if !client.is_authenticated() {
        bail!("Character isn't authenticated");
    }
    let character_lock = client
        .active_character
        .as_ref()
        .ok_or(anyhow!("Trying to obtain character, but no character is active for this client"))?
        .clone();

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

pub async fn handle_msg_move_worldport_ack(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;
    if !client.is_authenticated() {
        bail!("Character isn't authenticated");
    }
    let character_lock = client
        .active_character
        .as_ref()
        .ok_or(anyhow!("Trying to obtain character, but no character is active for this client"))?
        .clone();

    let character = character_lock.upgradable_read().await;

    if let TeleportationState::Executing(TeleportationDistance::Far(destination)) = character.teleportation_state.clone() {
        let mut character = RwLockUpgradableReadGuard::upgrade(character).await;

        let map = client_manager
            .world
            .get_instance_manager()
            .get_or_create_map(&(*character), destination.map)
            .await?;

        character.map = destination.map;
        character.set_position(&destination.into());
        character.reset_time_sync();
        character.send_packets_before_add_to_map(client_manager).await?;
        map.push_object(Arc::downgrade(&character_lock)).await;
        character.send_packets_after_add_to_map(client_manager).await?;

        character.teleportation_state = TeleportationState::None;
    }

    Ok(())
}
