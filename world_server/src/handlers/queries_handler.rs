use crate::character::character_manager::CharacterManager;
use crate::client::Client;
use crate::client_manager::ClientManager;
use crate::connection::events::ServerEvent;
use crate::prelude::*;
use crate::world::World;
use crate::{character::Character, world::prelude::GameObject};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use wow_world_messages::wrath::{
    CMSG_ITEM_NAME_QUERY, CMSG_ITEM_QUERY_SINGLE, CMSG_NAME_QUERY, CMSG_PLAYED_TIME, SMSG_ITEM_QUERY_SINGLE_RESPONSE, SMSG_NAME_QUERY_RESPONSE,
    SMSG_PLAYED_TIME, SMSG_QUERY_TIME_RESPONSE, SMSG_WORLD_STATE_UI_TIMER_UPDATE,
};

pub async fn handle_cmsg_played_time(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_PLAYED_TIME,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character_mut(guid)?;

    let (total_played_time, level_played_time) = {
        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
        let delta_seconds = unix_time - character.last_playtime_calculation_timestamp;
        character.seconds_played_total += delta_seconds;
        character.seconds_played_at_level += delta_seconds;
        character.last_playtime_calculation_timestamp = unix_time;
        (character.seconds_played_total, character.seconds_played_at_level)
    };

    let msg = SMSG_PLAYED_TIME {
        total_played_time,
        level_played_time,
        show_on_ui: packet.show_on_ui,
    };
    let event = ServerEvent::PlayedTime(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_query_time(client_manager: &ClientManager, client_id: SocketAddr) -> Result<()> {
    let client = client_manager.get_client(client_id)?;
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    let msg = SMSG_QUERY_TIME_RESPONSE {
        time: unix_time,
        time_until_daily_quest_reset: 0,
    };
    let event = ServerEvent::QueryTimeResponse(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_world_state_ui_timer_update(client_manager: &ClientManager, client_id: SocketAddr) -> Result<()> {
    let client = client_manager.get_client(client_id)?;
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    let msg = SMSG_WORLD_STATE_UI_TIMER_UPDATE { time: unix_time };
    let event = ServerEvent::WorldStateUiTimerUpdate(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_name_query(
    client_manager: &ClientManager,
    character_manager: &CharacterManager,
    client_id: SocketAddr,
    world: &World,
    packet: &CMSG_NAME_QUERY,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character(guid)?;

    //Stop early if we are requesting our own information
    if character.get_guid() == packet.guid {
        return send_name_query_response(client, character).await;
    }

    //We are requesting somebody else. Search the map
    if let Some(map) = world.get_instance_manager().try_get_map_for_character(character) {
        if map.find_character(packet.guid) {
            let found_character = character_manager.get_character(packet.guid)?;
            send_name_query_response(client, found_character).await?;
        } else {
            //This character is not on the same map as whoever requested it, so we do a lookup via
            //the client manager.
            if let Ok(found_client) = client_manager.find_client_from_active_character_guid(packet.guid) {
                let guid = found_client.get_active_character();
                let character = character_manager.get_character(guid)?;
                send_name_query_response(client, character).await?;
            }
        }
    } else {
        bail!("Character that requested cmsg_name_query has invalid instance_id");
    }
    Ok(())
}

async fn send_name_query_response(receiver: &Client, target_character: &Character) -> Result<()> {
    let msg = SMSG_NAME_QUERY_RESPONSE {
        guid: target_character.get_guid(),
        character_name: target_character.name.clone(),
        realm_name: String::new(),
        race: target_character.get_race(),
        class: target_character.get_class(),
        gender: target_character.get_gender(),
        has_declined_names: wow_world_messages::wrath::SMSG_NAME_QUERY_RESPONSE_DeclinedNames::No,
    };
    let event = ServerEvent::NameQueryResponse(msg);
    receiver.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_item_query_single(
    client_manager: &ClientManager,
    client_id: SocketAddr,
    _world: &World,
    packet: &CMSG_ITEM_QUERY_SINGLE,
) -> Result<()> {
    let client = client_manager.get_client(client_id)?;
    //TODO: use DB to lookup
    let item = wow_items::wrath::lookup_item(packet.item);
    let msg = match item {
        None => SMSG_ITEM_QUERY_SINGLE_RESPONSE {
            item: packet.item | 0x80000000,
            found: None,
        },
        Some(item) => wow_world_messages::wrath::item_to_query_response(item),
    };
    let event = ServerEvent::ItemQuerySingleResponse(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_item_name_query(
    client_manager: &ClientManager,
    client_id: SocketAddr,
    _world: &World,
    packet: &CMSG_ITEM_NAME_QUERY,
) -> Result<()> {
    //TODO: use DB to lookup
    let item = wow_items::wrath::lookup_item(packet.item);
    let client = client_manager.get_client(client_id)?;
    match item {
        Some(item) => {
            let msg = wow_world_messages::wrath::item_to_name_query_response(item);
            let event = ServerEvent::ItemNameQueryResponse(msg);
            client.connection_sender.send_async(event).await?;
            Ok(())
        }
        None => Err(anyhow!("Item {} not found for client {}", packet.item, client_id)),
    }
}
