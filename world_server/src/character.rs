use anyhow::{anyhow, Result};
use async_std::sync::RwLock;
use crate::guid::*;
use crate::client::Client;
use crate::ClientManager;
use crate::data_types::WorldZoneLocation;
use std::sync::{Weak};
use wrath_realm_db::{RealmDatabase};

pub struct Character
{
    pub guid: Guid,
    pub client: Weak<RwLock<Client>>,

    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation: f32,
    pub map: u32,
    pub bind_location: WorldZoneLocation,
}

impl Character
{
    pub async fn load_from_database(client: Weak<RwLock<Client>>, realm_database: &RealmDatabase, guid: Guid) -> Result<Self>
    {
        let db_entry = realm_database.get_character(guid.get_low_part()).await?;
        let bind_location = WorldZoneLocation
        {
            zone: db_entry.bind_zone as u32,
            map: db_entry.bind_map as u32,
            x: db_entry.bind_x,
            y: db_entry.bind_y,
            z: db_entry.bind_z,
        };

        let character_id = guid.get_low_part();
        let character_account_data = realm_database.get_character_account_data(character_id).await?;

        if character_account_data.len() == 0
        {
            crate::handlers::create_empty_character_account_data_rows(realm_database, character_id).await?;
        }

        Ok(Self {
            guid,
            client,
            x: db_entry.x,
            y: db_entry.y,
            z: db_entry.z,
            orientation: 0f32,
            map: db_entry.map as u32,
            bind_location
        })
    }

    pub async fn perform_login(&self, client_manager: &ClientManager) -> Result<()>
    {
        crate::handlers::send_verify_world(&self).await?;
        crate::handlers::send_dungeon_difficulty(&self).await?;
        crate::handlers::send_character_account_data_times(client_manager, &self).await?;
        {
            let client_lock = self.client.upgrade().ok_or_else(|| anyhow!("no client on character"))?;
            let client = client_lock.read().await;
            crate::handlers::send_voice_chat_status(&client, false).await?;
        }
        crate::handlers::send_bind_update(&self).await?;

        Ok(())
    }
}






