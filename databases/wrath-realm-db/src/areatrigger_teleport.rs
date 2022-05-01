use anyhow::Result;

#[derive(Debug)]
pub struct DBAreaTriggerTeleport {
    pub id: u32,
    pub name: Option<String>,
    pub required_level: u8,
    pub required_item: u32,
    pub required_item2: u32,
    pub heroic_key: u32,
    pub heroic_key2: u32,
    pub required_quest_done: u32,
    pub required_quest_done_heroic: u32,
    pub target_map: u16,
    pub target_position_x: f32,
    pub target_position_y: f32,
    pub target_position_z: f32,
    pub target_orientation: f32,
}

impl super::RealmDatabase {
    pub async fn get_areatrigger_teleport(&self, id: u32) -> Result<DBAreaTriggerTeleport> {
        let res = sqlx::query_as!(DBAreaTriggerTeleport, "SELECT * FROM areatrigger_teleport WHERE id = ?", id)
            .fetch_one(&self.connection_pool)
            .await?;

        Ok(res)
    }
}
