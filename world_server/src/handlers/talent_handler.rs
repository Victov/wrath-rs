use anyhow::Result;
use crate::packet::*;
use crate::character::*;
use crate::opcodes::Opcodes;
use podio::{WritePodExt, LittleEndian};

pub async fn send_talents_info(character: &Character) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_TALENTS_INFO, 20);
    
    //Cheese out and just say we have zero talent specs

    writer.write_u32::<LittleEndian>(0)?; //Free talent points
    writer.write_u8(0)?; //Number of talent specs
    writer.write_u8(0)?; //Id of current active spec

    send_packet_to_character(&character, header, &writer).await?;
    Ok(())
}
