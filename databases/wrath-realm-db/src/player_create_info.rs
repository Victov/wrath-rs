use anyhow::Result;

pub struct DBPlayerCreateInfo {
    pub race: u8,
    pub class: u8,
    pub map: u16,
    pub zone: u16,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub orientation: f32,
}

impl super::RealmDatabase {
    pub async fn get_player_create_info(&self, race: u8, class: u8) -> Result<DBPlayerCreateInfo> {
        let res = sqlx::query_as!(
            DBPlayerCreateInfo,
            "SELECT * FROM playercreateinfo WHERE race = ? AND class = ?",
            race,
            class
        )
        .fetch_one(&self.connection_pool)
        .await?;

        Ok(res)
    }
}
