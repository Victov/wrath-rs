use anyhow::Result;

pub struct DBItemInstance
{
    pub character_id : u32,
    pub slot_id : u8,
    pub item :u32,
    pub enchant: Option<u32>
}

impl super::RealmDatabase
{
    pub async fn get_all_character_equipment(&self, character_id : u32) -> Result<Vec<DBItemInstance>>
    {
        let res = sqlx::query_as!(
            DBItemInstance,
            "SELECT * FROM character_equipment WHERE character_id = ?",
            character_id
        )
        .fetch_all(&self.connection_pool)
        .await?;

        Ok(res)
    }
}