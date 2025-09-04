use crate::prelude::*;
use crate::{character::Character, connection::events::ServerEvent};

use wow_world_messages::wrath::{DungeonDifficulty, MSG_SET_DUNGEON_DIFFICULTY_Server};

pub async fn send_dungeon_difficulty(character: &Character) -> Result<()> {
    ServerEvent::SetDungeonDifficulty(MSG_SET_DUNGEON_DIFFICULTY_Server {
        difficulty: DungeonDifficulty::Normal,
        unknown1: 1,
        is_in_group: false,
    })
    .send_to_character(character)
    .await
}
