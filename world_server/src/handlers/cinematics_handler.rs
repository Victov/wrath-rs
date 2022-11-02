use crate::character::Character;
use crate::packet::ServerMessageExt;
use crate::prelude::*;
use wow_world_messages::wrath::{CinematicSequenceId, SMSG_TRIGGER_CINEMATIC};

pub async fn send_trigger_cinematic(character: &Character, cinematic_id: CinematicSequenceId) -> Result<()> {
    SMSG_TRIGGER_CINEMATIC {
        cinematic_sequence_id: cinematic_id,
    }
    .astd_send_to_character(character)
    .await
}
