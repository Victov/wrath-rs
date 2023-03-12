use crate::character::Character;
use crate::packet::ServerMessageExt;
use crate::prelude::*;

use wow_world_messages::wrath::{DungeonDifficulty, MSG_SET_DUNGEON_DIFFICULTY_Server};

pub async fn send_dungeon_difficulty(character: &Character) -> Result<()> {
    MSG_SET_DUNGEON_DIFFICULTY_Server {
        difficulty: DungeonDifficulty::Normal,
        unknown1: 1,
        is_in_group: false,
    }
    .astd_send_to_character(character)
    .await
}
