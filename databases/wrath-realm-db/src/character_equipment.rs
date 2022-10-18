use anyhow::Result;
use sqlx::{query, query_builder, MySql, QueryBuilder};

pub struct DBCharacterEquipment {
    pub character_id: u32,
    pub slot_id: u8,
    pub item: u32,
    pub enchant: Option<u32>,
}

#[derive(Debug)]
pub struct DBCharacterEquipmentDisplayInfo {
    pub slot_id: u8,
    pub inventory_type: Option<u8>,
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
"SELECT character_equipment.slot_id, character_equipment.enchant, item_template.inventory_type, item_template.displayid FROM character_equipment LEFT JOIN item_template ON character_equipment.item = item_template.id WHERE character_equipment.character_id = ?",
            character_id
        )
        .fetch_all(&self.connection_pool)
        .await?;

        Ok(res)
    }

    pub async fn give_character_start_equipment(&self, character_id: u32, item_ids: [i32; 24], inventory_type: [i32; 24]) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            //Cannot already have starting equipment
            assert_eq!(self.get_all_character_equipment(character_id).await?.len(), 0);
        }

        //Have to use slightly more complicated query builder syntax to bulk-insert.
        //Bulk insert is vastly faster than for-looping each item and "regular" inserting the items
        //one by one.
        let insert_iter = item_ids.iter().zip(inventory_type).filter_map(|(&item, slot_id)| {
            if item != -1 && slot_id != -1 {
                Some(DBCharacterEquipment {
                    character_id,
                    slot_id: slot_id as u8,
                    item: item as u32,
                    enchant: None,
                })
            } else {
                None
            }
        });

        let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new("INSERT INTO character_equipment (character_id, slot_id, item, enchant) ");
        query_builder.push_values(insert_iter, |mut b, item| {
            b.push_bind(item.character_id)
                .push_bind(item.slot_id)
                .push_bind(item.item)
                .push_bind(item.enchant);
        });

        let query = query_builder.build();
        query.execute(&self.connection_pool).await?;
        Ok(())
    }
}
