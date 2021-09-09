use crate::character::Character;
use crate::handlers::send_update;
use crate::updates::build_create_update_block_for_player;
use anyhow::{anyhow, Result};
use async_std::sync::RwLock;
use std::sync::{Arc, Weak};

pub struct MapManager {
    characters: RwLock<Vec<Weak<RwLock<Option<Character>>>>>,
}

impl MapManager {
    pub fn new() -> Self {
        Self {
            characters: RwLock::new(Vec::new()),
        }
    }

    pub async fn add_character_to_world(&self, character: &Character) -> Result<()> {
        let client_lock = character.client.upgrade().ok_or_else(|| anyhow!("Couldn't get client from character"))?;

        let client = client_lock.read().await;
        let weak_character = Arc::downgrade(&client.active_character);

        let mut write_characters = self.characters.write().await;
        write_characters.push(weak_character);

        {
            let mut update_data = character.update_data.lock().await;
            build_create_update_block_for_player(&mut update_data, &character, character)?;
        }
        send_update(character).await?;

        Ok(())
    }
}
