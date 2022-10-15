use crate::character::*;
use crate::client_manager::ClientManager;
use crate::packet::*;
use crate::prelude::*;
use wow_world_messages::wrath::Area;
use wow_world_messages::wrath::Object;
use wow_world_messages::wrath::WorldState;
use wow_world_messages::wrath::CMSG_TIME_SYNC_RESP;
use wow_world_messages::wrath::CMSG_ZONEUPDATE;
use wow_world_messages::wrath::SMSG_DESTROY_OBJECT;
use wow_world_messages::wrath::SMSG_INIT_WORLD_STATES;
use wow_world_messages::wrath::SMSG_TIME_SYNC_REQ;
use wow_world_messages::wrath::SMSG_UPDATE_OBJECT;
use wow_world_messages::wrath::SMSG_UPDATE_WORLD_STATE;

pub async fn handle_cmsg_zoneupdate(client_manager: &ClientManager, client_id: u64, packet: &CMSG_ZONEUPDATE) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let mut character = character_lock.write().await;
    (*character).zone_update(packet.area).await
}

pub async fn send_initial_world_states(character: &Character) -> Result<()> {
    SMSG_INIT_WORLD_STATES {
        map: character.map,
        area: character.area,
        sub_area: Area::NorthshireValley, //TODO: implement sub-areas

        //TODO figure out what these world states are and where to find non-hardcoded values
        //These need to end up in an enum
        states: vec![
            WorldState {
                state: 3191, //Arena season
                value: 1,
            },
            WorldState {
                state: 3901, //Arena season progress
                value: 1,
            },
        ],
    }
    .astd_send_to_character(character)
    .await
}

#[allow(dead_code)]
pub async fn send_world_state_update(character: &Character, world_state: WorldState) -> Result<()> {
    SMSG_UPDATE_WORLD_STATE { state: world_state }.astd_send_to_character(character).await
}

pub async fn send_smsg_update_objects(character: &Character, objects: Vec<Object>) -> Result<()> {
    SMSG_UPDATE_OBJECT { objects }.astd_send_to_character(character).await
}

pub async fn send_destroy_object(character: &Character, object_guid: Guid, is_death: bool) -> Result<()> {
    SMSG_DESTROY_OBJECT {
        guid: object_guid,
        target_died: is_death,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn send_time_sync(character: &Character) -> Result<()> {
    SMSG_TIME_SYNC_REQ {
        time_sync: character.time_sync_counter,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn handle_cmsg_time_sync_resp(client_manager: &ClientManager, client_id: u64, packet: &CMSG_TIME_SYNC_RESP) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let character = character_lock.read().await;
    if packet.time_sync != character.time_sync_counter {
        warn!(
            "Character {} has time sync issues. Reported: {}, expected {}, Could be cheating?",
            character.name, packet.time_sync, character.time_sync_counter
        );
    }
    Ok(())
}
