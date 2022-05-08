use anyhow::Result;

#[derive(Debug)]
pub struct DBAreaTriggerRestedZone {
    pub id: u32,
    pub name: Option<String>,
}

impl super::RealmDatabase {
    pub async fn get_areatrigger_rested_zone(&self, id: u32) -> Result<DBAreaTriggerRestedZone> {
        let res = sqlx::query_as!(DBAreaTriggerRestedZone, "SELECT * FROM areatrigger_restedzones WHERE id = ?", id)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(res)
    }
}
