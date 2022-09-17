use crate::character::*;
use crate::client::Client;
use crate::client_manager::ClientManager;
use crate::constants::inventory::*;
use crate::data::WritePositionAndOrientation;
use crate::packet::*;
use crate::prelude::*;
use crate::world::map_object::{MapObject, WorldObject};
use crate::world::World;
use podio::LittleEndian;
use std::collections::HashMap;
use std::convert::TryFrom;
use wow_world_messages::wrath::{Area, CharacterGear, Class, Gender, InventoryType, Map, Race, SMSG_CHAR_ENUM};
use wrath_realm_db::character::DBCharacterCreateParameters;

pub async fn handle_cmsg_char_enum(client_manager: &ClientManager, world: &World, client_id: u64) -> Result<()> {
    info!("into handle");
    let client = client_manager.get_authenticated_client(client_id).await?;

    let db_characters = world
        .get_realm_database()
        .get_characters_for_account(client.data.read().await.account_id.unwrap())
        .await?;

    let mut characters_to_send = Vec::<wow_world_messages::wrath::Character>::new();
    for character in db_characters {
        let equipment: HashMap<u8, wrath_realm_db::character_equipment::DBCharacterEquipmentDisplayInfo> = {
            let equipped_items = world.get_realm_database().get_all_character_equipment_display_info(character.id).await?;
            let mut hashmap = HashMap::default();
            for item in equipped_items {
                hashmap.insert(item.slot_id, item);
            }
            hashmap
        };

        let mut equipped_items_to_send = vec![];
        for equip_slot in EQUIPMENT_SLOTS_START..BAG_SLOTS_END + 1 {
            let gear = if let Some(equipped) = equipment.get(&equip_slot) {
                CharacterGear {
                    equipment_display_id: equipped.displayid.unwrap_or(0),
                    inventory_type: InventoryType::try_from(equipped.inventory_type.unwrap_or(0)).unwrap(),
                    enchantment: equipped.enchant.unwrap_or(0),
                }
            } else {
                CharacterGear {
                    equipment_display_id: 0,
                    inventory_type: InventoryType::Bag,
                    enchantment: 0,
                }
            };
            equipped_items_to_send.push(gear);
        }

        let character_flags = 0; //todo: stuff like being ghost, hide cloak, hide helmet, etc
        let is_first_login = false; //todo

        characters_to_send.push(wow_world_messages::wrath::Character {
            //TODO: restore functionality of the HighGuid that the non-wow_world_messages version
            //has
            //
            //let guid = Guid::new(character.id, HighGuid::Player);
            guid: wow_world_messages::Guid::from(character.id as u64),
            name: character.name.clone(),
            race: Race::try_from(character.race).unwrap_or(Race::Human),
            class: Class::try_from(character.class).unwrap_or(Class::Warrior),
            gender: Gender::try_from(character.gender).unwrap_or(Gender::Male),
            skin: character.skin_color,
            face: character.face,
            hair_style: character.hair_style,
            hair_color: character.hair_color,
            facial_hair: character.facial_style,
            level: character.level as u8,
            area: Area::try_from(character.zone as u32).unwrap_or(Area::ElwynnForest),
            map: Map::try_from(character.map as u32).unwrap_or(Map::EasternKingdoms),
            position: wow_world_messages::wrath::Vector3d {
                x: character.x,
                y: character.y,
                z: character.z,
            },
            guild_id: character.guild_id,
            flags: character_flags,
            recustomization_flags: 0,
            first_login: is_first_login,
            pet_display_id: 0,
            pet_level: 0,
            pet_family: 0,
            equipment: equipped_items_to_send.try_into().unwrap(),
        });

        info!("pushed 1 character: {}", character.name.clone());
    }

    SMSG_CHAR_ENUM {
        characters: characters_to_send,
    }
    .astd_send_to_client(client)
    .await?;

    info!("sent");
    Ok(())
}

#[allow(dead_code)]
enum CharacterCreateReponse {
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

/*
pub async fn handle_cmsg_char_create(client_manager: &ClientManager, world: &World, packet: &PacketToHandle) -> Result<()> {
    use std::io::BufRead;

    let mut reader = std::io::Cursor::new(&packet.payload);
    let client = client_manager.get_authenticated_client(packet.client_id).await?;

    let account_id = client.data.read().await.account_id.unwrap();

    let realm_db = world.get_realm_database();

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

        let player_create_info = realm_db.get_player_create_info(race, class).await?;

        let x = player_create_info.position_x;
        let y = player_create_info.position_y;
        let z = player_create_info.position_z;
        let o = player_create_info.orientation;
        let map = player_create_info.map;

        DBCharacterCreateParameters {
            account_id,
            name,
            race,
            class,
            gender,
            skin_color,
            face,
            hair_style,
            hair_color,
            facial_style,
            outfit,
            map,
            x,
            y,
            z,
            o,
        }
    };

    if !realm_db.is_character_name_available(&create_params.name).await? {
        send_char_create_reply(&client, CharacterCreateReponse::NameInUse).await?;
        return Ok(()); //this is a perfectly valid handling, not Err
    }

    let result = realm_db.create_character(&create_params).await;
    if result.is_err() {
        send_char_create_reply(&client, CharacterCreateReponse::Failed).await?;
        return Err(anyhow!("Failed to insert character into database"));
    }

    let realm_id = std::env::var("REALM_ID")?.parse()?;
    let num_chars = realm_db.get_num_characters_for_account(account_id).await?;
    client_manager
        .auth_db
        .set_num_characters_on_realm(account_id, realm_id, num_chars)
        .await?;

    send_char_create_reply(&client, CharacterCreateReponse::Success).await?;

    Ok(())
}

async fn send_char_create_reply(client: &Client, resp: CharacterCreateReponse) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_CHAR_CREATE, 1);
    writer.write_u8(resp as u8)?;
    send_packet(client, &header, &writer).await
}

pub async fn handle_cmsg_player_login(client_manager: &ClientManager, world: &World, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;

    let guid = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_guid::<LittleEndian>()?
    };

    client.load_and_set_active_character(client_manager, world, guid).await?;
    client.login_active_character(world).await?;

    Ok(())
}

pub async fn send_verify_world(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_LOGIN_VERIFY_WORLD, 20);
    writer.write_u32::<LittleEndian>(character.map)?;
    writer.write_position_and_orientation(character.get_position())?;
    send_packet_to_character(character, &header, &writer).await?;

    Ok(())
}

pub async fn send_bind_update(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_BINDPOINTUPDATE, 20);
    if let Some(bind_location) = &character.bind_location {
        writer.write_f32::<LittleEndian>(bind_location.x)?;
        writer.write_f32::<LittleEndian>(bind_location.y)?;
        writer.write_f32::<LittleEndian>(bind_location.z)?;
        writer.write_u32::<LittleEndian>(bind_location.map)?;
        writer.write_u32::<LittleEndian>(bind_location.zone)?;
        send_packet_to_character(character, &header, &writer).await?;
    } else {
        bail!("Requested to send Bind Update but character has no bind location")
    }

    Ok(())
}

pub async fn send_action_buttons(character: &Character) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_ACTION_BUTTONS, character.action_bar.data.len());
    writer.write_u8(0)?; //Talent specialization
    writer.write_all(&character.action_bar.data)?;

    send_packet_to_character(character, &header, &writer).await?;
    Ok(())
}

*/
