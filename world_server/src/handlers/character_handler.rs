use super::PacketToHandle;
use podio::{WritePodExt, LittleEndian};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use super::super::ClientManager;
use super::super::packet::*;
use super::*;
use std::io::Write;

pub async fn handle_cmsg_char_enum(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
{
    let client_lock = client_manager.get_client(packet.client_id).await?;
    
    if !client_lock.read().await.is_authenticated()
    {
        return Err(anyhow!("Not authenticated while retrieving character list"));
    }

    let (header, mut writer) = create_packet(Opcodes::SMSG_CHAR_ENUM, 40);

    let characters = client_manager.realm_db.get_characters_for_account(
        client_lock.read().await.account_id.unwrap()).await?;

    writer.write_u8(characters.len() as u8)?;

    for character in characters
    {
        let guid = Guid::new(character.id, 0, HighGuid::Player);
        let character_flags = 0; //todo: stuff like being ghost, hide cloak, hide helmet, etc
        let is_first_login = 0u8;

        writer.write_guid::<LittleEndian>(&guid)?;
        writer.write(character.name.as_bytes())?;
        writer.write_u8(0)?; //string terminator
        writer.write_u8(character.race)?;
        writer.write_u8(character.class)?;
        writer.write_u8(character.gender)?;
        writer.write_u8(character.skin_color)?;
        writer.write_u8(character.face)?;
        writer.write_u8(character.hair_style)?;
        writer.write_u8(character.hair_color)?;
        writer.write_u8(character.facial_style)?;
        writer.write_u8(character.level)?;
        writer.write_u32::<LittleEndian>(character.zone as u32)?;
        writer.write_u32::<LittleEndian>(character.map as u32)?;
        writer.write_f32::<LittleEndian>(character.x)?;
        writer.write_f32::<LittleEndian>(character.y)?;
        writer.write_f32::<LittleEndian>(character.z)?;
        writer.write_u32::<LittleEndian>(character.guild_id)?;
        writer.write_u32::<LittleEndian>(character_flags)?;
        writer.write_u8(is_first_login)?;
        writer.write_u32::<LittleEndian>(0)?;//pet display id
        writer.write_u32::<LittleEndian>(0)?;//pet level
        writer.write_u32::<LittleEndian>(0)?;//pet family 
        for _ in 0 .. 23 //inventory slot count
        {
            writer.write_u32::<LittleEndian>(0)?; //equipped item display id
            writer.write_u8(0)?; //inventory type
            writer.write_u32::<LittleEndian>(0)?; //enchant aura id
        }
    }

    {
        let client = client_lock.read().await;
        send_packet(&client, header, &writer).await?;
    }
    Ok(())
}
