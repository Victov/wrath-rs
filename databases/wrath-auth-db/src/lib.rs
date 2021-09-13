use anyhow::Result;
use sqlx::Row;
mod structs;
pub use structs::{DBAccount, DBAccountData, DBRealm};

pub struct AuthDatabase {
    connection_pool: sqlx::MySqlPool,
}

impl AuthDatabase {
    pub async fn new(conn_string: &String) -> Result<Self> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(conn_string.as_str())
            .await?;

        Ok(Self { connection_pool: pool })
    }

    pub async fn get_realm_bind_ip(&self, realm_id: i32) -> Result<String> {
        let bind_ip = sqlx::query("SELECT ip FROM realms WHERE id = ?")
            .bind(realm_id)
            .fetch_one(&self.connection_pool)
            .await?
            .try_get("ip")?;

        Ok(bind_ip)
    }

    pub async fn get_all_realms(&self) -> Result<Vec<DBRealm>> {
        Ok(sqlx::query_as!(DBRealm, "SELECT * FROM realms").fetch_all(&self.connection_pool).await?)
    }

    pub async fn set_realm_online_status(&self, realm_id: u32, online: bool) -> Result<()> {
        sqlx::query!("UPDATE realms SET online = ? WHERE id = ?", online as u8, realm_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn set_realm_population(&self, realm_id: u32, population: f32) -> Result<()> {
        sqlx::query!("UPDATE realms SET population = ? WHERE id = ?", population, realm_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn get_account_by_username(&self, username: &str) -> Result<DBAccount> {
        let acc = sqlx::query_as!(DBAccount, "SELECT * FROM accounts WHERE username = ?", username)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(acc)
    }

    pub async fn set_account_v_s(&self, account_id: u32, v: &str, s: &str) -> Result<()> {
        sqlx::query!("UPDATE accounts SET v = ?, s = ? WHERE id = ?", v, s, account_id)
            .execute(&self.connection_pool)
            .await?;

        Ok(())
    }

    pub async fn set_account_sessionkey(&self, username: &str, session_key: &str) -> Result<()> {
        sqlx::query!("UPDATE accounts SET sessionkey = ? WHERE username = ?;", session_key, username)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn get_account_data(&self, account_id: u32) -> Result<Vec<DBAccountData>> {
        let acc_data = sqlx::query_as!(DBAccountData, "SELECT * FROM account_data WHERE account_id = ?", account_id)
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(acc_data)
    }

    pub async fn create_account_data(&self, account_id: u32, data_type: u8) -> Result<()> {
        sqlx::query!(
            "INSERT INTO account_data (account_id, data_type, time, decompressed_size, data) VALUES (?,?, 0, 0, NULL)",
            account_id,
            data_type
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn update_account_data(&self, account_id: u32, new_time: u32, data_type: u8, new_length: u32, data: &[u8]) -> Result<()> {
        sqlx::query!(
            "UPDATE account_data SET decompressed_size = ?, data = ?, time = ? WHERE account_id = ? AND data_type = ?",
            new_length,
            data,
            new_time,
            account_id,
            data_type
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn get_num_characters_on_realm(&self, account_id: u32, realm_id: u32) -> Result<u8> {
        let res = sqlx::query!(
            "SELECT num_characters FROM realm_characters WHERE account_id = ? AND realm_id = ?",
            account_id,
            realm_id
        )
        .fetch_one(&self.connection_pool)
        .await;

        match res {
            Ok(row) => Ok(row.num_characters),
            Err(_) => Ok(0u8),
        }
    }

    pub async fn set_num_characters_on_realm(&self, account_id: u32, realm_id: u32, num_characters: u8) -> Result<()> {
        sqlx::query!("REPLACE INTO realm_characters VALUES (?, ?, ?)", account_id, realm_id, num_characters)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}
