use anyhow::{anyhow, Result};
use crate::packet::*;
use crate::character::*;
use crate::opcodes::Opcodes;
use crate::constants::social::*;
use podio::{WritePodExt, LittleEndian};

pub async fn send_contact_list(character: &Character, relation_types: &[RelationType]) -> Result<()>
{
    let mut mask: u32 = 0;
    for relation_type in relation_types.iter()
    {
        mask |= *relation_type as u32;
    }
    let (header, mut writer) = create_packet(Opcodes::SMSG_CONTACT_LIST, 8);

    if mask == 0
    {
        return Err(anyhow!("No relation types specified for sending contact list"));
    }

    writer.write_u32::<LittleEndian>(mask)?;
    writer.write_u32::<LittleEndian>(0)?; //zero friends, ignores, mutes, etc
    
    send_packet_to_character(&character, header, &writer).await?;
    Ok(())
}


