use crate::packet::ServerMessageExt;
use crate::prelude::*;
use crate::{character::Character, client_manager::ClientManager};
use wow_world_messages::wrath::{CinematicSequenceId, CMSG_COMPLETE_CINEMATIC, CMSG_NEXT_CINEMATIC_CAMERA, SMSG_TRIGGER_CINEMATIC};

pub async fn send_trigger_cinematic(character: &Character, cinematic_id: CinematicSequenceId) -> Result<()> {
    SMSG_TRIGGER_CINEMATIC {
        cinematic_sequence_id: cinematic_id,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn handle_csmg_next_cinematic_camera(client_manager: &ClientManager, client_id: u64, _packet: &CMSG_NEXT_CINEMATIC_CAMERA) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let mut character = character_lock.write().await;
    character.handle_cinematic_next_camera()
}

pub async fn handle_csmg_complete_cinematic(client_manager: &ClientManager, client_id: u64, _packet: &CMSG_COMPLETE_CINEMATIC) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let mut character = character_lock.write().await;
    character.handle_cinematic_ended()
}
