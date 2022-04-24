use crate::character::*;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::prelude::*;
use podio::{LittleEndian, WritePodExt};

pub async fn send_faction_list(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_INITIALIZE_FACTIONS, 500);
    writer.write_u32::<LittleEndian>(128)?; //Number of factions

    //https://github.com/WCell/WCell/blob/master/Services/WCell.RealmServer/Handlers/FactionHandler.cs#L110
    //write zeroes if we don't have that faction yet.
    //So maybe it's valid to not know a single faction?
    //Send all zeroes for now
    for _ in 0..128 {
        writer.write_u8(0)?;
        writer.write_u32::<LittleEndian>(0)?;
    }
    send_packet_to_character(character, &header, &writer).await?;
    Ok(())
}
