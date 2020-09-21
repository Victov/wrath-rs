use anyhow::Result;
use super::guid::*;
use super::client::Client;
use wrath_realm_db::{RealmDatabase};
use std::sync::{Weak};
use async_std::sync::RwLock;

pub struct Character
{
    pub guid: Guid,
    pub client: Weak<RwLock<Client>>,

    pub x:f32,
    pub y:f32,
    pub z:f32,
    pub orientation:f32,
    pub map:u32,
}

impl Character
{
    pub async fn load_from_database(client: Weak<RwLock<Client>>, realm_database: &RealmDatabase, guid: Guid) -> Result<Self>
    {
        let db_entry = realm_database.get_character(guid.get_low_part()).await?;

        Ok(Self {
            guid,
            client,
            x: db_entry.x,
            y: db_entry.y,
            z: db_entry.z,
            orientation: 0f32,
            map: db_entry.map as u32,
        })
    }
}






