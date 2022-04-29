use crate::character::*;
use crate::client_manager::ClientManager;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use crate::world::value_fields::HasValueFields;
use crate::world::value_fields::PlayerFields;
use podio::{LittleEndian, ReadPodExt, WritePodExt};
use std::sync::Arc;

pub async fn send_initial_spells(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_INITIAL_SPELLS, 500);
    writer.write_u8(0)?;

    let num_spells: u16 = 2;
    writer.write_u16::<LittleEndian>(num_spells)?;
    for i in 0..num_spells {
        writer.write_u32::<LittleEndian>(70282 + i as u32)?; //fireball and hellfire, randomly taken for testing
        writer.write_u16::<LittleEndian>(0)?;
    }

    //Cheese out and don't send any cooldowns
    writer.write_u16::<LittleEndian>(0)?; //Number of cooldowns

    send_packet_to_character(character, &header, &writer).await?;
    Ok(())
}

pub async fn send_aura_update_all(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_AURA_UPDATE_ALL, 200);
    writer.write_guid_compressed(&character.guid)?;

    send_packet_to_character(character, &header, &writer).await?;
    Ok(())
}

pub async fn handle_cmsg_set_actionbar_toggles(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let actionbar = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_u8()?
    };

    let character_lock = client.get_active_character().await?;

    let mut character = character_lock.write().await;
    (*character).set_byte(PlayerFields::Bytes as usize, 2, actionbar)?;

    Ok(())
}
