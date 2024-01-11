use std::time::Duration;

use anyhow::Result;
use entity::{realm_characters, realms};
use sea_orm::{
    sea_query::Expr, Condition, ConnectOptions, Database, DatabaseConnection, DbBackend, EntityTrait, FromQueryResult, JoinType, QuerySelect,
    QueryTrait, RelationTrait,
};
mod structs;
pub use structs::{DBAccount, DBAccountData, DBRealm, DBRealmWithNumCharacters};
mod entity;
pub use entity::{accounts::Entity as Accounts, realm_characters::Entity as RealmCharacters, realms::Entity as Realms};

pub struct AuthDatabase {
    // I kept both the DatabaseConnection and the sqlx::MySqlPool so you can compare them
    connection_pool: sqlx::MySqlPool,
    database: DatabaseConnection,
}

impl AuthDatabase {
    pub async fn new(conn_string: &String, timeout: Duration) -> Result<Self> {
        let mut options = ConnectOptions::new(conn_string);

        options
            .max_connections(5)
            .acquire_timeout(timeout)
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Debug);

        let database = Database::connect(options).await?;

        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(timeout)
            .connect(conn_string.as_str())
            .await?;

        Ok(Self {
            connection_pool: pool,
            database,
        })
    }

    pub async fn get_realm_bind_ip(&self, realm_id: i32) -> Result<String> {
        // simple example using the Entity API to find a single record
        let bind_ip = Realms::find_by_id(realm_id as u32).one(&self.database).await?.unwrap().ip;

        Ok(bind_ip)
    }

    pub async fn get_all_realms_with_num_characters(&self, account_id: u32) -> Result<Vec<DBRealmWithNumCharacters>> {
        Ok(sqlx::query_as!(
            DBRealmWithNumCharacters,
            "SELECT r.*, rc.num_characters as num_characters FROM realms r
            LEFT JOIN realm_characters rc ON rc.account_id = ? AND rc.realm_id = r.id",
            account_id
        )
        .fetch_all(&self.connection_pool)
        .await?)
    }

    // Here I kept both methods so you could compare or even interchange when running the code
    pub async fn sea_get_all_realms_with_num_characters(&self, account_id: u32) -> Result<Vec<DBRealmWithNumCharacters>> {
        let query_statement = Realms::find()
            .column_as(realm_characters::Column::NumCharacters, "num_characters")
            .join(
                JoinType::LeftJoin,
                realms::Relation::RealmCharacters.def().on_condition(move |_left, right| {
                    Condition::all().add(Expr::col((right.clone(), realm_characters::Column::AccountId)).eq(account_id))
                }),
            )
            .build(DbBackend::MySql);

        // prints the generated query
        println!("Query statement: {:?}", query_statement.to_string());

        let fetch_data = DBRealmWithNumCharacters::find_by_statement(query_statement.clone())
            .all(&self.database)
            .await?;

        Ok(fetch_data)
    }

    pub async fn get_all_realms(&self) -> Result<Vec<DBRealm>> {
        Ok(sqlx::query_as!(DBRealm, "SELECT * FROM realms").fetch_all(&self.connection_pool).await?)
    }

    pub async fn create_account(&self, username: &str, v: &str, s: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO `accounts` (`username`, `v`, `s`) VALUES (?, ?, ?)",
            username.to_lowercase(),
            v,
            s
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
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

    pub async fn get_account_by_username(&self, username: &str) -> Result<Option<DBAccount>> {
        let acc = sqlx::query_as!(DBAccount, "SELECT * FROM accounts WHERE username = ?", username)
            .fetch_optional(&self.connection_pool)
            .await?;
        Ok(acc)
    }

    pub async fn set_account_sessionkey(&self, username: &str, session_key: &str) -> Result<()> {
        sqlx::query!("UPDATE accounts SET sessionkey = ? WHERE username = ?;", session_key, username)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn set_account_ban_status(&self, username: &str, banned: bool) -> Result<()> {
        let banned_int = banned as u8;
        sqlx::query!("UPDATE `accounts` SET banned = ? WHERE username = ?;", banned_int, username)
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

    pub async fn get_account_data_of_type(&self, account_id: u32, data_type: u8) -> Result<DBAccountData> {
        Ok(sqlx::query_as!(
            DBAccountData,
            "SELECT * FROM account_data WHERE account_id = ? AND data_type = ?",
            account_id,
            data_type
        )
        .fetch_one(&self.connection_pool)
        .await?)
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
