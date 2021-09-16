use crate::character::*;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::prelude::*;
use podio::{LittleEndian, WritePodExt};

pub async fn send_initial_world_states(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_INIT_WORLD_STATES, 8);
    writer.write_u32::<LittleEndian>(character.map)?;
    writer.write_u32::<LittleEndian>(0)?; //zone
    writer.write_u32::<LittleEndian>(0)?; //area

    //hardcode for now, should be dynamic
    writer.write_u16::<LittleEndian>(2)?; //count of world states

    writer.write_u32::<LittleEndian>(3191)?; //arena season world state id
    writer.write_u32::<LittleEndian>(1)?;
    writer.write_u32::<LittleEndian>(3901)?; //arena progress world state id
    writer.write_u32::<LittleEndian>(1)?;

    send_packet_to_character(&character, header, &writer).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn send_world_state_update(character: &Character, world_state: u32, value: u32) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_UPDATE_WORLD_STATE, 8);
    writer.write_u32::<LittleEndian>(world_state)?;
    writer.write_u32::<LittleEndian>(value)?;

    send_packet_to_character(&character, header, &writer).await?;
    Ok(())
}

pub async fn send_update_packet(character: &Character, num_blocks: u32, data: &[u8]) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_UPDATE_OBJECT, 8);
    writer.write_u32::<LittleEndian>(num_blocks)?;
    {
        use std::io::Write;
        writer.write(data)?;
    }

    send_packet_to_character(character, header, &writer).await
}
