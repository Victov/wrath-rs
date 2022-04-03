use std::time::Duration;

use anyhow::Result;

pub mod character;
pub mod character_account_data;

pub struct RealmDatabase {
    connection_pool: sqlx::MySqlPool,
}

impl RealmDatabase {
    pub async fn new(conn_string: &String, timeout: Duration) -> Result<Self> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect_timeout(timeout)
            .connect(conn_string.as_str())
            .await?;

        Ok(Self { connection_pool: pool })
    }
}
