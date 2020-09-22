//Temporarily allow unused imports until this file has some more beef to it
//Then we know what we don't need and can delete them
#![allow(unused_imports)]

use anyhow::{Result, anyhow};
use crate::packet_handler::PacketToHandle;
use crate::character::Character;
use crate::packet::*;
use crate::opcodes::Opcodes;
use podio::{WritePodExt, ReadPodExt, LittleEndian};
use std::sync::Arc;
use std::io::Write;
use wrath_realm_db::character::DBCharacterCreateParameters;


pub async fn send_dungeon_difficulty(character: &Character) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::MSG_SET_DUNGEON_DIFFICULTY, 12);
    
    //TODO: get dungeon difficulty from character instead of hardcoded
    //TODO: handle being in a group

    writer.write_u32::<LittleEndian>(0)?; //0: N, 1:HC
    writer.write_u32::<LittleEndian>(1)?; //unknown?
    writer.write_u32::<LittleEndian>(0)?; //bool IsInGroup

    send_packet_to_character(&character, header, &writer).await?;

    Ok(())
}
