use std::net::SocketAddr;

use crate::character::character_manager::CharacterManager;
use crate::character::*;
use crate::client_manager::ClientManager;
use crate::connection::events::ServerEvent;
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

pub async fn handle_cmsg_zoneupdate(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_ZONEUPDATE,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character_mut(guid)?;
    character.zone_update(packet.area).await?;
    Ok(())
}

pub async fn send_initial_world_states(character: &Character) -> Result<()> {
    let msg = SMSG_INIT_WORLD_STATES {
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
    };
    ServerEvent::InitWorldStates(msg).send_to_character(character).await
}

#[allow(dead_code)]
pub async fn send_world_state_update(character: &Character, world_state: WorldState) -> Result<()> {
    ServerEvent::UpdateWorldState(SMSG_UPDATE_WORLD_STATE { state: world_state })
        .send_to_character(character)
        .await
}

pub async fn send_smsg_update_objects(character: &Character, objects: Vec<Object>) -> Result<()> {
    ServerEvent::UpdateObject(SMSG_UPDATE_OBJECT { objects })
        .send_to_character(character)
        .await
}

pub async fn send_destroy_object(character: &Character, object_guid: Guid, is_death: bool) -> Result<()> {
    ServerEvent::DestroyObject(SMSG_DESTROY_OBJECT {
        guid: object_guid,
        target_died: is_death,
    })
    .send_to_character(character)
    .await
}

pub async fn send_time_sync(character: &Character) -> Result<()> {
    ServerEvent::TimeSyncReq(SMSG_TIME_SYNC_REQ {
        time_sync: character.time_sync_counter,
    })
    .send_to_character(character)
    .await
}

pub async fn handle_cmsg_time_sync_resp(
    client_manager: &ClientManager,
    character_manager: &CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_TIME_SYNC_RESP,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character(guid)?;

    if packet.time_sync != character.time_sync_counter {
        warn!(
            "Character {} has time sync issues. Reported: {}, expected {}, Could be cheating?",
            character.name, packet.time_sync, character.time_sync_counter
        );
    }
    Ok(())
}
