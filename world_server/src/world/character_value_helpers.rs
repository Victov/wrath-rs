use super::prelude::{stand_state::UnitStandState, UnitValueHelpers};
use crate::prelude::*;
use crate::{character::Character, handlers};

#[async_trait::async_trait]
pub trait CharacterValueHelpers: UnitValueHelpers {
    fn as_character(&self) -> &Character;

    async fn set_character_stand_state(&mut self, state: UnitStandState) -> Result<()> {
        self.set_stand_state(state)?;
        let as_character = self.as_character();
        handlers::send_smsg_stand_state_update(as_character, state).await
    }

    async fn set_rooted(&self, rooted: bool) -> Result<()> {
        let as_character = self.as_character();
        if rooted {
            handlers::send_smsg_force_move_root(as_character).await
        } else {
            handlers::send_smsg_force_move_unroot(as_character).await
        }
    }
}

impl CharacterValueHelpers for Character {
    fn as_character(&self) -> &Character {
        self
    }
}
