use anyhow::Result;
use crate::packet::*;
use crate::character::*;
use crate::opcodes::Opcodes;
use podio::{WritePodExt, LittleEndian};

pub async fn send_initial_spells(character: &Character) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_INITIAL_SPELLS, 500);
    writer.write_u8(0)?;
    
    let num_spells: u16 = 2;
    for i in 0 .. num_spells
    {
        writer.write_u32::<LittleEndian>(70282 + i as u32)?; //fireball and hellfire, randomly taken for testing
        writer.write_u16::<LittleEndian>(0)?;
    }

    //Cheese out and don't send any cooldowns
    writer.write_u16::<LittleEndian>(0)?; //Number of cooldowns

    send_packet_to_character(&character, header, &writer).await?;
    Ok(())
}
