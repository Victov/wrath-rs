use crate::item::Item;
use crate::prelude::*;
use crate::world::prelude::*;
use async_std::sync::RwLock;
use std::sync::Arc;

impl super::Character {
    pub(super) async fn load_equipment_from_database(&mut self, world: &World) -> Result<()> {
        let equipped_items = world.get_realm_database().get_all_character_equipment(self.guid.get_low_part()).await?;
        for equipped_item in equipped_items {
            let item = Item::new(equipped_item.item, self, world).await?;
            self.equipped_items.push(Arc::new(RwLock::new(item)));
        }

        Ok(())
    }

    pub(super) async fn equipment_on_added_to_map(&mut self, map: &MapManager) -> Result<()> {
        for item in &self.equipped_items {
            map.push_object(Arc::downgrade(item)).await;
        }
        Ok(())
    }
}
