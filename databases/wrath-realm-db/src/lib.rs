use anyhow::Result;
use std::time::Duration;

pub mod areatrigger_restedzone;
pub mod areatrigger_teleport;
pub mod character;
pub mod character_account_data;
pub mod character_equipment;
pub mod item_instance;
pub mod item_template;
pub mod player_create_info;

pub struct RealmDatabase {
    connection_pool: sqlx::MySqlPool,
}

impl RealmDatabase {
    pub async fn new(conn_string: &str, timeout: Duration) -> Result<Self> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(timeout)
            .connect(conn_string)
            .await?;

        Ok(Self { connection_pool: pool })
    }
}
