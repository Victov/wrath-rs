use anyhow::Result;

pub struct RealmDatabase
{
    connection_pool : sqlx::MySqlPool,
}

impl RealmDatabase
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
}
