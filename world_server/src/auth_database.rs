use anyhow::Result;
use sqlx::{Row};

pub struct AuthDatabase
{
    connection_pool : sqlx::MySqlPool,
}

impl AuthDatabase
{
    pub async fn new(conn_string : &String) -> Result<Self>
    {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(conn_string.as_str())
            .await?;

        Ok(Self
           {
               connection_pool : pool,
           })
    }

    pub async fn get_realm_bind_ip(&self, realm_id : i32) -> Result<String>
    {
        let bind_ip = sqlx::query("SELECT ip FROM realms WHERE id = ?")
            .bind(realm_id)
            .fetch_one(&self.connection_pool)
            .await?
            .try_get("ip")?;

        Ok(bind_ip)   
    }
}
