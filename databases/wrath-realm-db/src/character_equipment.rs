use anyhow::Result;

pub struct DBCharacterEquipment {
    pub character_id: u32,
    pub slot_id: u8,
    pub item: u32,
    pub enchant: Option<u32>,
}

#[derive(Debug)]
pub struct DBCharacterEquipmentDisplayInfo {
    pub slot_id: u8,
    pub enchant: Option<u32>,
    pub displayid: Option<u32>,
}

impl super::RealmDatabase {
    pub async fn get_all_character_equipment(&self, character_id: u32) -> Result<Vec<DBCharacterEquipment>> {
        let res = sqlx::query_as!(
            DBCharacterEquipment,
            "SELECT * FROM character_equipment WHERE character_id = ?",
            character_id
        )
        .fetch_all(&self.connection_pool)
        .await?;

        Ok(res)
    }

    pub async fn get_all_character_equipment_display_info(&self, character_id: u32) -> Result<Vec<DBCharacterEquipmentDisplayInfo>> {
        let res = sqlx::query_as!(
            DBCharacterEquipmentDisplayInfo,
"SELECT character_equipment.slot_id, character_equipment.enchant, item_template.displayid FROM character_equipment LEFT JOIN item_template ON character_equipment.item = item_template.id WHERE character_equipment.character_id = ?",
            character_id
        )
        .fetch_all(&self.connection_pool)
        .await?;

        Ok(res)
    }
}
