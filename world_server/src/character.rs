use anyhow::Result;
use async_std::sync::{Mutex, RwLock};
use crate::constants::social::RelationType;
use crate::guid::*;
use crate::data_types::{WorldZoneLocation, ActionBar, TutorialFlags};
use crate::client::Client;
use crate::ClientManager;
use crate::updates::{constants::{ObjectUpdateFlags, ObjectType}, Updateable, UpdateData};
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
    pub update_data: Mutex<UpdateData>,
}

impl Updateable for Character
{
    fn get_guid(&self) -> &Guid
    {
        &self.guid
    }

    fn get_update_flags(&self) -> u32
    {
        ObjectUpdateFlags::Living as u32 | ObjectUpdateFlags::Position as u32 
    }

    fn get_object_type(&self) -> u32
    {
        ObjectType::Player as u32
    }

    fn get_object_type_mask(&self) -> u32
    {
        1 << ObjectType::Player as u32
    }
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
            update_data: Mutex::new(UpdateData::new()),
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
        crate::handlers::send_initial_spells(&self).await?;
        crate::handlers::send_talents_info(&self).await?;
        crate::handlers::send_aura_update_all(&self).await?;
        crate::handlers::send_contact_list(&self, &[RelationType::Friend, RelationType::Muted, RelationType::Ignore]).await?;
        crate::handlers::send_initial_world_states(&self).await?;
        
        client_manager.world.map_manager.add_character_to_world(self).await?;

        Ok(())
    }
}






