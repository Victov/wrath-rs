use anyhow::Result;
use sqlx::{Row};
mod structs;
pub use structs::{DBRealm, DBAccount};

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

    pub async fn get_all_realms(&self) -> Result<Vec<DBRealm>>
    {
        Ok(sqlx::query_as!(DBRealm, "SELECT * FROM realms")
            .fetch_all(&self.connection_pool)
            .await?)
    }

    pub async fn set_realm_online_status(&self, realm_id: u32, online: bool) -> Result<()>
    {
        sqlx::query!("UPDATE realms SET online = ? WHERE id = ?", online as u8, realm_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn set_realm_population(&self, realm_id: u32, population: f32) -> Result<()>
    {
        sqlx::query!("UPDATE realms SET population = ? WHERE id = ?", population, realm_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn get_account_by_username(&self, username: &str) -> Result<DBAccount>
    {
        let acc = sqlx::query_as!(DBAccount, "SELECT * FROM accounts WHERE username = ?", username)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(acc)
    }

    pub async fn set_account_v_s(&self, account_id: u32, v: &str, s: &str) -> Result<()>
    {
        sqlx::query!("UPDATE accounts SET v = ?, s = ? WHERE id = ?", v, s, account_id)
            .execute(&self.connection_pool)
            .await?;

        Ok(())
    }
    
    pub async fn set_account_sessionkey(&self, username : &str, session_key : &str) -> Result<()>
    {
        sqlx::query!("UPDATE accounts SET sessionkey = ? WHERE username = ?;", session_key, username)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}