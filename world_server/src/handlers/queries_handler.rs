use crate::client::Client;
use crate::client_manager::ClientManager;
use crate::packet::*;
use crate::prelude::*;
use crate::world::World;
use crate::{character::Character, world::prelude::GameObject};
use std::time::{SystemTime, UNIX_EPOCH};
use wow_world_messages::wrath::{
    CMSG_NAME_QUERY, CMSG_PLAYED_TIME, CMSG_QUERY_TIME, CMSG_WORLD_STATE_UI_TIMER_UPDATE, SMSG_NAME_QUERY_RESPONSE, SMSG_PLAYED_TIME,
    SMSG_QUERY_TIME_RESPONSE, SMSG_WORLD_STATE_UI_TIMER_UPDATE,
};

pub async fn handle_cmsg_played_time(client_manager: &ClientManager, client_id: u64, packet: &CMSG_PLAYED_TIME) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let (total_played_time, level_played_time) = {
        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
        let mut character = character_lock.write().await;
        let delta_seconds = unix_time - character.last_playtime_calculation_timestamp;
        character.seconds_played_total += delta_seconds;
        character.seconds_played_at_level += delta_seconds;
        character.last_playtime_calculation_timestamp = unix_time;
        (character.seconds_played_total, character.seconds_played_at_level)
    };

    SMSG_PLAYED_TIME {
        total_played_time,
        level_played_time,
        show_on_ui: packet.show_on_ui,
    }
    .astd_send_to_client(client)
    .await
}

pub async fn handle_cmsg_query_time(client_manager: &ClientManager, client_id: u64, _packet: &CMSG_QUERY_TIME) -> Result<()> {
    let client = client_manager.get_client(client_id).await?;
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    SMSG_QUERY_TIME_RESPONSE {
        time: unix_time,
        time_until_daily_quest_reset: 0,
    }
    .astd_send_to_client(client)
    .await
}

pub async fn handle_cmsg_world_state_ui_timer_update(
    client_manager: &ClientManager,
    client_id: u64,
    _packet: &CMSG_WORLD_STATE_UI_TIMER_UPDATE,
) -> Result<()> {
    let client = client_manager.get_client(client_id).await?;
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    SMSG_WORLD_STATE_UI_TIMER_UPDATE { time: unix_time }.astd_send_to_client(client).await
}

pub async fn handle_cmsg_name_query(client_manager: &ClientManager, client_id: u64, world: &World, packet: &CMSG_NAME_QUERY) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    //Stop early if we are requesting our own information
    let character = character_lock.read().await;
    if character.get_guid() == packet.guid {
        return send_name_query_response(&*client, &*character).await;
    }

    //We are requesting somebody else. Search the map
    if let Some(map) = world.get_instance_manager().try_get_map_for_character(&*character).await {
        if let Some(found_character_lock) = map.try_get_object(packet.guid).await.and_then(|a| a.upgrade()) {
            if let Some(found_character) = found_character_lock.read().await.as_character() {
                send_name_query_response(&*client, found_character).await?;
            } else {
                bail!("There was a cmsg_name_query for a found object, but it was not a character");
            }
        } else {
            bail!("There was a cmsg_name_query for a guid that is not on the same map as the requester")
        }
    } else {
        bail!("Character that requested cmsg_name_query has invalid instance_id");
    }
    Ok(())
}

async fn send_name_query_response(receiver: &Client, target_character: &Character) -> Result<()> {
    SMSG_NAME_QUERY_RESPONSE {
        guid: target_character.get_guid(),
        character_name: target_character.name.clone(),
        realm_name: String::new(),
        race: target_character.get_race(),
        class: target_character.get_class(),
        gender: target_character.get_gender(),
        has_declined_names: wow_world_messages::wrath::SMSG_NAME_QUERY_RESPONSE_HasDeclinedNames::No,
    }
    .astd_send_to_client(receiver)
    .await
}
