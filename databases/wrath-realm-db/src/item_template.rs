use anyhow::Result;

pub struct DBItemTemplate {
    pub id: u32,
    pub name: String,
    pub displayid: u32,
    pub inventory_type: u8,
}

impl super::RealmDatabase {
    pub async fn get_item_template(&self, item_id: u32) -> Result<DBItemTemplate> {
        let res = sqlx::query_as!(
            DBItemTemplate,
            "SELECT `id`, `name`, `displayid`,`inventory_type` FROM item_template WHERE id = ?",
            item_id,
        )
        .fetch_one(&self.connection_pool)
        .await?;

        Ok(res)
    }
}
