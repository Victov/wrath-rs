use crate::character::Character;
use crate::opcodes::Opcodes;
use crate::packet::*;
use anyhow::Result;
use podio::{LittleEndian, WritePodExt};
use std::io::Write;

pub async fn send_update(character: &Character) -> Result<()> {
    let update_data = character.update_data.lock().await;

    let (header, mut writer) = create_packet(Opcodes::SMSG_UPDATE_OBJECT, 128);

    writer.write_u32::<LittleEndian>(update_data.block_count as u32)?;
    writer.write(&update_data.data)?;

    send_packet_to_character(&character, header, &writer).await?;

    Ok(())
}
