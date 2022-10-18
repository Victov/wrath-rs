use crate::character::Character;
use crate::client_manager::ClientManager;
use crate::constants::inventory::*;
use crate::data::DataStorage;
use crate::packet::*;
use crate::prelude::*;
use crate::world::prelude::GameObject;
use crate::world::World;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use wow_dbc::DbcTable;
use wow_world_messages::wrath::WorldResult;
use wow_world_messages::wrath::CMSG_CHAR_CREATE;
use wow_world_messages::wrath::CMSG_PLAYER_LOGIN;
use wow_world_messages::wrath::SMSG_ACTION_BUTTONS;
use wow_world_messages::wrath::SMSG_BINDPOINTUPDATE;
use wow_world_messages::wrath::SMSG_CHAR_CREATE;
use wow_world_messages::wrath::SMSG_LOGIN_VERIFY_WORLD;
use wow_world_messages::wrath::{Area, CharacterGear, Class, Gender, InventoryType, Map, Race, SMSG_CHAR_ENUM};
use wrath_realm_db::character::DBCharacterCreateParameters;
use wrath_realm_db::RealmDatabase;

pub async fn handle_cmsg_char_enum(client_manager: &ClientManager, world: &World, client_id: u64) -> Result<()> {
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
        let first_login = false; //todo

        assert_eq!(equipped_items_to_send.len(), 23);

        characters_to_send.push(wow_world_messages::wrath::Character {
            //TODO: restore functionality of the HighGuid that the non-wow_world_messages version
            //has
            //
            //let guid = Guid::new(character.id, HighGuid::Player);
            guid: wow_world_messages::Guid::from(character.id as u64),
            name: character.name,
            race: Race::try_from(character.race).unwrap_or(Race::Human),
            class: Class::try_from(character.class).unwrap_or(Class::Warrior),
            gender: Gender::try_from(character.gender).unwrap_or(Gender::Male),
            skin: character.skin_color,
            face: character.face,
            hair_style: character.hair_style,
            hair_color: character.hair_color,
            facial_hair: character.facial_style,
            level: character.level as u8,
            area: Area::try_from(character.zone as u32).unwrap_or(Area::NorthshireValley),
            map: Map::try_from(character.map as u32).unwrap_or(Map::EasternKingdoms),
            position: wow_world_messages::wrath::Vector3d {
                x: character.x,
                y: character.y,
                z: character.z,
            },
            guild_id: character.guild_id,
            flags: character_flags,
            recustomization_flags: 0,
            first_login,
            pet_display_id: 0,
            pet_level: 0,
            pet_family: 0,
            equipment: equipped_items_to_send.try_into().unwrap(),
        });
    }

    SMSG_CHAR_ENUM {
        characters: characters_to_send,
    }
    .astd_send_to_client(client)
    .await?;

    Ok(())
}

pub async fn handle_cmsg_char_create(client_manager: &ClientManager, client_id: u64, world: &World, data: &CMSG_CHAR_CREATE) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let account_id = client.data.read().await.account_id.unwrap();
    let realm_db = world.get_realm_database();

    let create_params = {
        let player_create_info = realm_db.get_player_create_info(data.race.as_int(), data.class.as_int()).await?;

        let x = player_create_info.position_x;
        let y = player_create_info.position_y;
        let z = player_create_info.position_z;
        let o = player_create_info.orientation;
        let map = player_create_info.map;

        DBCharacterCreateParameters {
            account_id,
            name: data.name.clone(),
            race: data.race.as_int(),
            class: data.class.as_int(),
            gender: data.gender.as_int(),
            skin_color: data.skin_color,
            face: data.face,
            hair_style: data.hair_style,
            hair_color: data.hair_color,
            facial_style: data.facial_hair,
            outfit: CMSG_CHAR_CREATE::OUTFIT_ID_VALUE,
            map,
            x,
            y,
            z,
            o,
        }
    };

    if !realm_db.is_character_name_available(&create_params.name).await? {
        SMSG_CHAR_CREATE {
            result: WorldResult::CharCreateNameInUse,
        }
        .astd_send_to_client(client)
        .await?;

        return Ok(());
    }

    let insert_result = realm_db.create_character(&create_params).await;
    if insert_result.is_err() {
        SMSG_CHAR_CREATE {
            result: WorldResult::CharCreateFailed,
        }
        .astd_send_to_client(client)
        .await?;

        return Err(anyhow!("Failed to insert character into database"));
    }

    //Safe to unwrap since we caught is_err() just above
    let inserted_character_id = insert_result.unwrap();

    let realm_id = std::env::var("REALM_ID")?.parse()?;
    let num_chars = realm_db.get_num_characters_for_account(account_id).await?;
    client_manager
        .auth_db
        .set_num_characters_on_realm(account_id, realm_id, num_chars)
        .await?;

    give_character_start_equipment(
        inserted_character_id as u32,
        data.race,
        data.class,
        data.gender,
        &client_manager.data_storage,
        &realm_db,
    )
    .await?;

    SMSG_CHAR_CREATE {
        result: WorldResult::CharCreateSuccess,
    }
    .astd_send_to_client(client)
    .await
}

async fn give_character_start_equipment(
    character_id: u32,
    race: Race,
    class: Class,
    gender: Gender,
    data_storage: &DataStorage,
    realm_db: &RealmDatabase,
) -> Result<()> {
    let start_outfit_info = data_storage
        .get_dbc_char_start_outfit()?
        .rows()
        .iter()
        .find(|row| row.class_id.id == class.as_int() as i32 && row.race_id.id == race.as_int() as i32 && row.sex_id == gender.as_int() as i8)
        .ok_or_else(|| anyhow!("Class/Race/Gender combination not found for starting outfit"))?;

    info!("Start equipment: {:?}", start_outfit_info);
    realm_db
        .give_character_start_equipment(character_id, start_outfit_info.item_id, start_outfit_info.inventory_type)
        .await?;

    Ok(())
}

pub async fn handle_cmsg_player_login(client_manager: &ClientManager, world: &World, client_id: u64, data: &CMSG_PLAYER_LOGIN) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let guid = data.guid;

    client.load_and_set_active_character(client_manager, world, guid).await?;
    client.login_active_character(world).await?;

    Ok(())
}

pub async fn send_verify_world(character: &Character) -> Result<()> {
    let position = character
        .get_position()
        .ok_or_else(|| anyhow!("Characters should always have a position"))?;

    SMSG_LOGIN_VERIFY_WORLD {
        map: character.map,
        position: position.position,
        orientation: position.orientation,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn send_bind_update(character: &Character) -> Result<()> {
    if let Some(bind_location) = &character.bind_location {
        SMSG_BINDPOINTUPDATE {
            position: bind_location.position,
            map: bind_location.map,
            area: bind_location.area,
        }
        .astd_send_to_character(character)
        .await
    } else {
        bail!("Requested to send Bind Update but character has no bind location")
    }
}

pub async fn send_action_buttons(character: &Character) -> Result<()> {
    SMSG_ACTION_BUTTONS {
        behavior: wow_world_messages::wrath::SMSG_ACTION_BUTTONS_ActionBarBehavior::Initial {
            data: character.action_bar.data,
        },
    }
    .astd_send_to_character(character)
    .await
}
