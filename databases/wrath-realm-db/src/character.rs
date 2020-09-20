use anyhow::Result;

pub struct DBCharacter
{
    pub id: u32,
    pub account_id: u32,
    pub name: String,
    pub race: u8,
    pub class: u8,
    pub gender: u8,
    pub skin_color: u8,
    pub face: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub facial_style: u8,
    pub player_flags: u32,
    pub at_login_flags: u16,
    pub zone: u16,
    pub level: u8,
    pub map: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub guild_id: u32,
}

impl super::RealmDatabase
{
    pub async fn get_characters_for_account(&self, account_id: u32) -> Result<Vec<DBCharacter>>
    {
        let res = sqlx::query_as!(DBCharacter, "SELECT * FROM characters WHERE account_id = ?", account_id)
            .fetch_all(&self.connection_pool)
            .await?;

        Ok(res)
    }
}
