use anyhow::{Result, anyhow};
use crate::packet_handler::PacketToHandle;
use crate::packet::*;
use crate::character::*;
use crate::opcodes::Opcodes;
use crate::guid::{Guid, ReadGuid, WriteGuid, HighGuid};
use crate::client_manager::ClientManager;
use crate::client::Client;
use podio::{WritePodExt, ReadPodExt, LittleEndian};
use std::sync::Arc;
use std::io::Write;
use wrath_realm_db::character::DBCharacterCreateParameters;

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
        writer.write(character.name.as_bytes())?; //Investigate: Don't need to manually write string terminator here?
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
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u8(is_first_login)?;
        writer.write_u32::<LittleEndian>(0)?;//pet display id
        writer.write_u32::<LittleEndian>(0)?;//pet level
        writer.write_u32::<LittleEndian>(0)?;//pet family
        for _ in 0 .. 23u8 //inventory slot count
        {
            writer.write_u32::<LittleEndian>(0)?; //equipped item display id
            writer.write_u8(0)?; //inventory type
            writer.write_u32::<LittleEndian>(0)?; //enchant aura id
        }
    }

    let client = client_lock.read().await;
    send_packet(&client, header, &writer).await?;
    Ok(())
}

#[allow(dead_code)]
enum CharacterCreateReponse
{
    InProgres = 0x2E,
    Success = 0x2F,
    Error = 0x30,
    Failed = 0x31,
    NameInUse = 0x32,
    Disable = 0x33,
    PvpTeamsViolation = 0x34,
    ServerLimit = 0x35,
    AccountLimit = 0x36,
    ServerQueue = 0x37,
    OnlyExisting = 0x38,
    Expansion = 0x39,
    ExpansionClass = 0x3A,
    LevelRequirement = 0x3B,
    UniqueClassLimit = 0x3C,
    CharacterInGuild = 0x3D,
    RestrictedRaceClass = 0x3E,
    CharacterChooseRace = 0x3F,
    CharacterArenaLeader = 0x40,
    CharacterDeleteMail = 0x41,
    CharacterSwapFaction = 0x42,
    CharacterRaceOnly = 0x43,
    CharacterGoldLimit = 0x44,
    ForceLogin = 0x45,
}

pub async fn handle_cmsg_char_create(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
{
    use std::io::BufRead;

    let mut reader = std::io::Cursor::new(&packet.payload);
    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;

    if !client.is_authenticated()
    {
        return Err(anyhow!("Unauthenticated client tried to create character"));
    }

    let account_id = client.account_id.unwrap();

    let create_params = {
        let mut name = Vec::<u8>::new();
        reader.read_until(0u8, &mut name)?;
        let name = String::from_utf8(name)?;
        let race = reader.read_u8()?;
        let class = reader.read_u8()?;
        let gender = reader.read_u8()?;
        let skin_color = reader.read_u8()?;
        let face = reader.read_u8()?;
        let hair_style = reader.read_u8()?;
        let hair_color = reader.read_u8()?;
        let facial_style = reader.read_u8()?;
        let outfit = reader.read_u8()?;

        DBCharacterCreateParameters
        {
            account_id, name, race, class, gender, skin_color, face, hair_style, hair_color, facial_style, outfit
        }
    };

    let realm_db = &client_manager.realm_db;
    if !realm_db.is_character_name_available(&create_params.name).await?
    {
        send_char_create_reply(&client, CharacterCreateReponse::NameInUse).await?;
        return Ok(()); //this is a perfectly valid handling, not Err
    }

    let result = realm_db.create_character(&create_params).await;
    if result.is_err()
    {
        send_char_create_reply(&client, CharacterCreateReponse::Failed).await?;
        return Err(anyhow!("Failed to insert character into database"));
    }

    let realm_id = std::env::var("REALM_ID")?.parse()?;
    let num_chars = realm_db.get_num_characters_for_account(account_id).await?;
    client_manager.auth_db.set_num_characters_on_realm(account_id, realm_id, num_chars).await?;

    send_char_create_reply(&client, CharacterCreateReponse::Success).await?;

    Ok(())
}

async fn send_char_create_reply(client: &Client, resp: CharacterCreateReponse) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_CHAR_CREATE, 1);
    writer.write_u8(resp as u8)?;
    send_packet(client, header, &writer).await
}

pub async fn handle_cmsg_player_login(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
{
    let client_lock = client_manager.get_client(packet.client_id).await?;
    if !client_lock.read().await.is_authenticated()
    {
        return Err(anyhow!("Trying to login character on client that isn't authenticated"));
    }

    let guid =
    {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_guid::<LittleEndian>()?
    };

    let client = client_lock.read().await;
    client.login_character(client_manager, guid).await?;

    Ok(())
}

pub async fn send_verify_world(character: &Character) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_LOGIN_VERIFY_WORLD, 20);
    writer.write_u32::<LittleEndian>(character.map)?;
    writer.write_f32::<LittleEndian>(character.x)?;
    writer.write_f32::<LittleEndian>(character.y)?;
    writer.write_f32::<LittleEndian>(character.z)?;
    writer.write_f32::<LittleEndian>(character.orientation)?;

    send_packet_to_character(&character, header, &writer).await?;

    Ok(())
}

pub async fn send_bind_update(character: &Character) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_BINDPOINTUPDATE, 20);
    writer.write_f32::<LittleEndian>(character.bind_location.x)?;
    writer.write_f32::<LittleEndian>(character.bind_location.y)?;
    writer.write_f32::<LittleEndian>(character.bind_location.z)?;
    writer.write_u32::<LittleEndian>(character.bind_location.map)?;
    writer.write_u32::<LittleEndian>(character.bind_location.zone)?;

    send_packet_to_character(&character, header, &writer).await?;

    Ok(())
}

pub async fn send_action_buttons(character: &Character) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_ACTION_BUTTONS, character.action_bar.data.len());
    writer.write_u8(0)?; //Talent specialization
    writer.write(&character.action_bar.data)?;

    send_packet_to_character(&character, header, &writer).await?;
    Ok(())
}
