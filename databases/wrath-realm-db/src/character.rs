use anyhow::Result;

pub struct DBCharacter {
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
    pub o: f32,
    pub instance_id: u32,
    pub bind_zone: u16,
    pub bind_map: u16,
    pub bind_x: f32,
    pub bind_y: f32,
    pub bind_z: f32,
    pub guild_id: u32,
    pub tutorial_data: Vec<u8>,
    pub playtime_total: u32,
    pub playtime_level: u32,
}

pub struct DBSaveableCharacterParameters {
    pub id: u32,
    pub level: u8,
    pub map: u16,
    pub zone: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub o: f32,
    pub instance_id: u32,
    pub bind_zone: u16,
    pub bind_map: u16,
    pub bind_x: f32,
    pub bind_y: f32,
    pub bind_z: f32,
    pub guild_id: u32,
    pub tutorial_data: Vec<u8>,
    pub playtime_total: u32,
    pub playtime_level: u32,
}

pub struct DBCharacterCreateParameters {
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
    pub outfit: u8,
    pub map: u16,
    pub zone: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub o: f32,
}

impl super::RealmDatabase {
    pub async fn get_characters_for_account(&self, account_id: u32) -> Result<Vec<DBCharacter>> {
        let res = sqlx::query_as!(DBCharacter, "SELECT * FROM characters WHERE account_id = ?", account_id)
            .fetch_all(&self.connection_pool)
            .await?;

        Ok(res)
    }

    pub async fn get_num_characters_for_account(&self, account_id: u32) -> Result<u8> {
        let res = sqlx::query!("SELECT count(*) as cnt FROM characters WHERE account_id = ?", account_id)
            .fetch_one(&self.connection_pool)
            .await?;

        Ok(res.cnt as u8)
    }

    pub async fn is_character_name_available(&self, name: &str) -> Result<bool> {
        let res = sqlx::query!("SELECT count(*) AS cnt FROM characters WHERE name = ?", name)
            .fetch_one(&self.connection_pool)
            .await;

        match res {
            Ok(result) => Ok(result.cnt == 0),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    pub async fn create_character(&self, params: &DBCharacterCreateParameters) -> Result<u64> {
        let empty_tutorial_data: Vec<u8> = Vec::new();

        let result = sqlx::query!("INSERT INTO characters (`account_id`, `name`, `race`, `class`, `gender`, `skin_color`, `face`, `hair_style`, `hair_color`, `facial_style`, `zone`, `map`, `x`, `y`, `z`, `o`, `tutorial_data`) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?,?, ?, ?, ?, ?, ?, ?, ?);",
        params.account_id,
        params.name,
        params.race,
        params.class,
        params.gender,
        params.skin_color,
        params.face,
        params.hair_style,
        params.hair_color,
        params.facial_style,
        params.zone,
        params.map,
        params.x,
        params.y,
        params.z,
        params.o,
        empty_tutorial_data,
        )
            .execute(&self.connection_pool)
            .await?;

        Ok(result.last_insert_id())
    }

    pub async fn update_character_data(&self, params: &DBSaveableCharacterParameters) -> Result<()> {
        sqlx::query!(
            "UPDATE characters SET level = ?, map = ?, zone = ?, x = ?, y = ?, z = ?, o = ?, instance_id = ?, bind_zone = ?, bind_map = ?, bind_x = ?, bind_y = ?, bind_z = ?, guild_id = ?, tutorial_data = ?, playtime_total = ?, playtime_level = ? WHERE id = ?",
            params.level,
            params.map,
            params.zone,
            params.x,
            params.y,
            params.z,
            params.o,
            params.instance_id,
            params.bind_zone,
            params.bind_map,
            params.bind_x,
            params.bind_y,
            params.bind_z,
            params.guild_id,
            params.tutorial_data,
            params.playtime_total,
            params.playtime_level,
            params.id
        )
            .execute(&self.connection_pool)
            .await?;

        Ok(())
    }

    pub async fn get_character(&self, character_id: u32) -> Result<DBCharacter> {
        let res = sqlx::query_as!(DBCharacter, "SELECT * FROM characters WHERE id = ?", character_id)
            .fetch_one(&self.connection_pool)
            .await?;

        Ok(res)
    }

    pub async fn delete_character(&self, character_id: u32, account_id: u32) -> Result<bool> {
        let res = sqlx::query_as!(
            DBCharacter,
            "DELETE FROM characters WHERE id = ? AND account_id = ?",
            character_id,
            account_id
        )
        .execute(&self.connection_pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }
}
