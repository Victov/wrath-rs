use crate::character::character_inventory::SimpleCharacterInventory;
use crate::character::character_inventory::SimpleItemDescription;
use crate::character::character_inventory::INVENTORY_SLOT_BAG_0;
use crate::character::character_manager::CharacterManager;
use crate::character::Character;
use crate::client_manager::ClientManager;
use crate::connection::events::ServerEvent;
use crate::constants::inventory::*;
use crate::data::DataStorage;
use crate::prelude::*;
use crate::world::prelude::GameObject;
use crate::world::World;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::net::SocketAddr;
use wow_dbc::DbcTable;
use wow_world_messages::wrath::WorldResult;
use wow_world_messages::wrath::CMSG_AUTOEQUIP_ITEM;
use wow_world_messages::wrath::CMSG_CHAR_CREATE;
use wow_world_messages::wrath::CMSG_CHAR_DELETE;
use wow_world_messages::wrath::CMSG_PLAYER_LOGIN;
use wow_world_messages::wrath::CMSG_STANDSTATECHANGE;
use wow_world_messages::wrath::CMSG_SWAP_INV_ITEM;
use wow_world_messages::wrath::SMSG_ACTION_BUTTONS;
use wow_world_messages::wrath::SMSG_BINDPOINTUPDATE;
use wow_world_messages::wrath::SMSG_CHAR_CREATE;
use wow_world_messages::wrath::SMSG_CHAR_DELETE;
use wow_world_messages::wrath::SMSG_LOGIN_VERIFY_WORLD;
use wow_world_messages::wrath::{Area, CharacterGear, Class, Gender, InventoryType, Map, Race, SMSG_CHAR_ENUM};
use wrath_realm_db::character::DBCharacterCreateParameters;
use wrath_realm_db::RealmDatabase;

pub async fn handle_cmsg_char_enum(client_manager: &ClientManager, world: &World, client_id: SocketAddr) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;

    let db_characters = world.get_realm_database().get_characters_for_account(client.data.account_id).await?;

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
        let first_login = character.playtime_total == 0;

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
            level: character.level.into(),
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
            pet_level: 0.into(),
            pet_family: wow_world_messages::wrath::CreatureFamily::None,
            equipment: equipped_items_to_send.try_into().unwrap(),
        });
    }

    let msg = SMSG_CHAR_ENUM {
        characters: characters_to_send,
    };
    let event = ServerEvent::CharEnum(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_char_create(client_manager: &ClientManager, client_id: SocketAddr, world: &World, data: &CMSG_CHAR_CREATE) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let account_id = client.data.account_id;
    let realm_db = world.get_realm_database();

    let create_params = {
        let player_create_info = realm_db.get_player_create_info(data.race.as_int(), data.class.as_int()).await?;

        let x = player_create_info.position_x;
        let y = player_create_info.position_y;
        let z = player_create_info.position_z;
        let o = player_create_info.orientation;
        let map = player_create_info.map;
        let zone = player_create_info.zone;

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
            zone,
            x,
            y,
            z,
            o,
        }
    };

    if !realm_db.is_character_name_available(&create_params.name).await? {
        let msg = SMSG_CHAR_CREATE {
            result: WorldResult::CharCreateNameInUse,
        };
        let event = ServerEvent::CharCreate(msg);
        client.connection_sender.send_async(event).await?;
        return Ok(());
    }

    let insert_result = realm_db.create_character(&create_params).await;
    if insert_result.is_err() {
        let msg = SMSG_CHAR_CREATE {
            result: WorldResult::CharCreateFailed,
        };
        let event = ServerEvent::CharCreate(msg);
        client.connection_sender.send_async(event).await?;

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

    let msg = SMSG_CHAR_CREATE {
        result: WorldResult::CharCreateSuccess,
    };
    let event = ServerEvent::CharCreate(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_char_delete(client_manager: &ClientManager, client_id: SocketAddr, world: &World, data: &CMSG_CHAR_DELETE) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let account_id = client.data.account_id;
    let realm_db = world.get_realm_database();

    let character_id = data.guid.guid() as u32;

    let result = match realm_db.delete_character(character_id, account_id).await {
        Ok(_) => WorldResult::CharDeleteSuccess,
        // TODO: Handle guild leader and arena captain failure cases.
        Err(_) => WorldResult::CharDeleteFailed,
    };

    let msg = SMSG_CHAR_DELETE { result };
    let event = ServerEvent::CharDelete(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
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

    let mut dummy_inventory = SimpleCharacterInventory::new();
    let slot_ids: Vec<i32> = start_outfit_info
        .inventory_type
        .iter()
        .zip(start_outfit_info.item_id)
        .map(|(&inv_type, item_id)| {
            let inventory_type: InventoryType = (inv_type as u8).try_into()?;

            let sid = SimpleItemDescription {
                item_id: item_id as u32,
                inventory_type,
            };

            dummy_inventory.try_insert_item(sid).map(|res| res.to_owned())
        })
        .map(|res: anyhow::Result<EquipmentSlot, anyhow::Error>| if let Ok(s) = res { s as i32 } else { -1 })
        .collect();

    realm_db
        .give_character_start_equipment(character_id, start_outfit_info.item_id, slot_ids)
        .await
}

pub async fn handle_cmsg_player_login(
    client_manager: &mut ClientManager,
    character_manager: &mut CharacterManager,
    world: &mut World,
    client_id: SocketAddr,
    data: &CMSG_PLAYER_LOGIN,
) -> Result<()> {
    let connection_sender = client_manager.get_authenticated_client(client_id)?.connection_sender.clone();
    let character = Character::load(connection_sender, data.guid, world, &client_manager.data_storage).await?;
    character_manager.add_character(character);
    let client = client_manager.get_authenticated_client_mut(client_id).await?;
    client.set_active_character(data.guid);
    client.login_active_character(world, character_manager).await
}

pub async fn handle_cmsg_player_logout(
    client_manager: &mut ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character_id = client.get_active_character();
    let character = character_manager.get_character_mut(character_id)?;
    character.try_logout().await?;
    Ok(())
}

pub async fn handle_cmsg_standstate_change(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    data: &CMSG_STANDSTATECHANGE,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;
    character.set_stand_state(data.animation_state).await
}

pub async fn send_verify_world(character: &Character) -> Result<()> {
    let position = character
        .get_position()
        .ok_or_else(|| anyhow!("Characters should always have a position"))?;

    let msg = SMSG_LOGIN_VERIFY_WORLD {
        map: character.map,
        position: position.position,
        orientation: position.orientation,
    };
    ServerEvent::LoginVerifyWorld(msg).send_to_character(character).await
}

pub async fn send_bind_update(character: &Character) -> Result<()> {
    if let Some(bind_location) = &character.bind_location {
        let msg = SMSG_BINDPOINTUPDATE {
            position: bind_location.position,
            map: bind_location.map,
            area: bind_location.area,
        };
        ServerEvent::BindPointUpdate(msg).send_to_character(character).await
    } else {
        bail!("Requested to send Bind Update but character has no bind location")
    }
}

pub async fn send_action_buttons(character: &Character) -> Result<()> {
    let msg = SMSG_ACTION_BUTTONS {
        behavior: wow_world_messages::wrath::SMSG_ACTION_BUTTONS_ActionBarBehavior::Initial {
            data: character.action_bar.data,
        },
    };
    ServerEvent::ActionButtons(msg).send_to_character(character).await
}

pub async fn handle_cmsg_swap_inv_item(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    _world: &World,
    client_id: SocketAddr,
    data: &CMSG_SWAP_INV_ITEM,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;

    //TODO: Add checks here
    let src = data.destination_slot.as_int();
    let dst = data.source_slot.as_int();
    let dst_item = character.set_item(None, (dst, INVENTORY_SLOT_BAG_0))?;
    let src_item = character.set_item(dst_item, (src, INVENTORY_SLOT_BAG_0))?;
    character.set_item(src_item, (dst, INVENTORY_SLOT_BAG_0))?;

    Ok(())
}

pub async fn handle_cmsg_autoequip_item(
    client_manager: &mut ClientManager,
    character_manager: &mut CharacterManager,
    _world: &World,
    client_id: SocketAddr,
    data: &CMSG_AUTOEQUIP_ITEM,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;

    let previously_equipped_item = character.auto_equip_item_from_bag((data.source_slot, data.source_bag))?;

    //The item that we had equipped (may be None) now goes into that slot
    character.set_item(previously_equipped_item, (data.source_slot, data.source_bag))?;
    Ok(())
}
