use crate::character::*;
use crate::client_manager::ClientManager;
use crate::packet::*;
use crate::prelude::*;
use wow_world_messages::wrath::Object;
use wow_world_messages::wrath::CMSG_ZONEUPDATE;
use wow_world_messages::wrath::SMSG_TIME_SYNC_REQ;
use wow_world_messages::wrath::SMSG_UPDATE_OBJECT;

pub async fn handle_cmsg_zoneupdate(client_manager: &ClientManager, client_id: u64, packet: &CMSG_ZONEUPDATE) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let mut character = character_lock.write().await;
    (*character).zone_update(packet.area).await
}

/*
pub async fn send_initial_world_states(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_INIT_WORLD_STATES, 8);
    writer.write_u32::<LittleEndian>(character.map)?;
    writer.write_u32::<LittleEndian>(character.zone)?;
    writer.write_u32::<LittleEndian>(0)?; //area

    //hardcode for now, should be dynamic
    writer.write_u16::<LittleEndian>(2)?; //count of world states

    writer.write_u32::<LittleEndian>(3191)?; //arena season world state id
    writer.write_u32::<LittleEndian>(1)?;
    writer.write_u32::<LittleEndian>(3901)?; //arena progress world state id
    writer.write_u32::<LittleEndian>(1)?;

    send_packet_to_character(character, &header, &writer).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn send_world_state_update(character: &Character, world_state: u32, value: u32) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_UPDATE_WORLD_STATE, 8);
    writer.write_u32::<LittleEndian>(world_state)?;
    writer.write_u32::<LittleEndian>(value)?;

    send_packet_to_character(character, &header, &writer).await?;
    Ok(())
}
*/

pub async fn send_smsg_update_objects(character: &Character, objects: Vec<Object>) -> Result<()> {
    SMSG_UPDATE_OBJECT { objects }.astd_send_to_character(character).await
}

/*
pub async fn send_destroy_object(character: &Character, object_guid: Guid, is_death: bool) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_DESTROY_OBJECT, 9);
    writer.write_guid_compressed(object_guid)?;
    writer.write_u8(is_death as u8)?;
    send_packet_to_character(character, &header, &writer).await
}
*/

pub async fn send_time_sync(character: &Character) -> Result<()> {
    SMSG_TIME_SYNC_REQ {
        time_sync: character.time_sync_counter,
    }
    .astd_send_to_character(character)
    .await
}

/*
pub async fn handle_cmsg_time_sync_resp(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let character_lock = client.get_active_character().await?;

    let mut reader = std::io::Cursor::new(&packet.payload);
    let counter = reader.read_u32::<LittleEndian>()?;
    let _client_ticks = reader.read_u32::<LittleEndian>()?;

    let character = character_lock.read().await;
    if counter != character.time_sync_counter {
        warn!(
            "Character {} has time sync issues. Reported: {}, expected {}, Could be cheating?",
            character.name, counter, character.time_sync_counter
        );
    }
    Ok(())
}

*/
