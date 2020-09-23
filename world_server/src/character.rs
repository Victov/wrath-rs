use anyhow::Result;
use async_std::sync::RwLock;
use crate::guid::*;
use crate::data_types::{WorldZoneLocation, ActionBar, TutorialFlags};
use crate::client::Client;
use crate::ClientManager;
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
    pub tutorial_flags: TutorialFlags,
    pub action_bar: ActionBar,
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

        let tutorial_flags = TutorialFlags::from_database_entry(&db_entry)?;

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
            bind_location,
            tutorial_flags,
            action_bar: ActionBar::new(), //TODO: store/read from database
        })
    }

    pub async fn perform_login(&self, client_manager: &ClientManager) -> Result<()>
    {
        crate::handlers::send_verify_world(&self).await?;
        crate::handlers::send_dungeon_difficulty(&self).await?;
        crate::handlers::send_character_account_data_times(client_manager, &self).await?;
        crate::handlers::send_voice_chat_status(&self, false).await?;
        crate::handlers::send_bind_update(&self).await?;
        crate::handlers::send_tutorial_flags(&self).await?;
        crate::handlers::send_login_set_time_speed(&self).await?;
        crate::handlers::send_action_buttons(&self).await?;
        crate::handlers::send_faction_list(&self).await?;

        Ok(())
    }
}






