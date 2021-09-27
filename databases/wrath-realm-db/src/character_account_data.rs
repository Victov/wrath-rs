use anyhow::Result;

pub struct DBCharacterAccountData {
    pub character_id: u32,
    pub data_type: u8,
    pub time: u64,
    pub decompressed_size: u32,
    pub data: Option<Vec<u8>>,
}

impl super::RealmDatabase {
    pub async fn get_character_account_data(&self, character_id: u32) -> Result<Vec<DBCharacterAccountData>> {
        let res = sqlx::query_as!(
            DBCharacterAccountData,
            "SELECT * FROM character_account_data WHERE character_id = ?",
            character_id
        )
        .fetch_all(&self.connection_pool)
        .await?;

        Ok(res)
    }

    pub async fn get_character_account_data_of_type(&self, character_id: u32, data_type: u8) -> Result<DBCharacterAccountData> {
        Ok(sqlx::query_as!(
            DBCharacterAccountData,
            "SELECT * FROM character_account_data WHERE character_id = ? AND data_type = ?",
            character_id,
            data_type
        )
        .fetch_one(&self.connection_pool)
        .await?)
    }

    pub async fn create_character_account_data(&self, character_id: u32, data_type: u8) -> Result<()> {
        sqlx::query!(
            "INSERT INTO character_account_data (character_id, data_type, time, decompressed_size, data) VALUES (?,?, 0, 0, NULL)",
            character_id,
            data_type
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn update_character_account_data(&self, character_id: u32, new_time: u32, data_type: u8, new_length: u32, data: &[u8]) -> Result<()> {
        sqlx::query!(
            "UPDATE character_account_data SET decompressed_size = ?, data = ?, time = ? WHERE character_id = ? AND data_type = ?",
            new_length,
            data,
            new_time,
            character_id,
            data_type
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}
