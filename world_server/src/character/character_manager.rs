use std::collections::HashMap;

use anyhow::{anyhow, Result};
use tracing::info;
use wow_world_messages::Guid;

use crate::{character::Character, world::prelude::GameObject};

#[derive(Default)]
pub struct CharacterManager {
    characters: HashMap<Guid, Character>,
}

impl CharacterManager {
    pub fn new() -> Self {
        Self { characters: HashMap::new() }
    }

    pub fn add_character(&mut self, character: Character) {
        let guid = character.get_guid();
        self.characters.insert(guid, character);
    }

    pub fn find_character(&self, guid: Guid) -> Option<&Character> {
        self.characters.get(&guid)
    }

    pub fn find_character_mut(&mut self, guid: Guid) -> Option<&mut Character> {
        self.characters.get_mut(&guid)
    }

    pub fn get_character(&self, guid: Guid) -> Result<&Character> {
        self.characters
            .get(&guid)
            .ok_or(anyhow!("Character with guid {} not found in character manager", guid))
    }

    pub fn get_character_mut(&mut self, guid: Guid) -> Result<&mut Character> {
        self.characters
            .get_mut(&guid)
            .ok_or(anyhow!("Character with guid {} not found in character manager", guid))
    }

    pub fn remove_character(&mut self, guid: Guid) {
        info!("Character with guid {} removed from character manager", guid);
        self.characters.remove(&guid);
    }
}
