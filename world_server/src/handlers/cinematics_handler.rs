use std::net::SocketAddr;

use crate::character::character_manager::CharacterManager;
use crate::connection::events::ServerEvent;
use crate::prelude::*;
use crate::{character::Character, client_manager::ClientManager};
use wow_world_messages::wrath::{CinematicSequenceId, SMSG_TRIGGER_CINEMATIC};

pub async fn send_trigger_cinematic(character: &Character, cinematic_id: CinematicSequenceId) -> Result<()> {
    let msg = SMSG_TRIGGER_CINEMATIC {
        cinematic_sequence_id: cinematic_id,
    };
    ServerEvent::TriggerCinematic(msg).send_to_character(character).await
}

pub async fn handle_csmg_next_cinematic_camera(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;
    character.handle_cinematic_next_camera()
}

pub async fn handle_csmg_complete_cinematic(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;
    character.handle_cinematic_ended()
}
