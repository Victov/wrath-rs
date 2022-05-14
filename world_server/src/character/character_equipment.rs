use crate::constants::inventory::*;
use crate::item::Item;
use crate::prelude::*;
use crate::world::prelude::*;
use async_std::sync::RwLock;
use std::sync::Arc;

impl super::Character {
    pub(super) async fn load_equipment_from_database(&mut self, world: &World) -> Result<()> {
        let equipped_items = world.get_realm_database().get_all_character_equipment(self.guid.get_low_part()).await?;
        for equipped_item in equipped_items {
            let mut item = Item::new(equipped_item.item, self, world).await?;
            self.set_equipped_item_visual_flags(equipped_item.slot_id, &mut item).await?;
            self.equipped_items.push(Arc::new(RwLock::new(item)));
        }

        Ok(())
    }

    pub(super) async fn push_create_blocks_for_items(&mut self, map: &MapManager) -> Result<()> {
        for item in self.equipped_items.clone() {
            let (num, mut buf) = {
                let item = item.read().await;
                build_create_update_block_for_player(self, &*item)?
            };
            map.push_object(Arc::downgrade(&item)).await;
            self.push_update_block(&mut buf, num);
        }
        Ok(())
    }

    pub(super) async fn set_equipped_item_visual_flags(&mut self, slot: u8, item: &mut Item) -> Result<()> {
        //self.set_field_guid(PlayerFields::InvSlotHead as usize + (slot as usize * 2), item.get_guid())?;
        item.set_field_guid(ItemFields::Contained as usize, self.get_guid())?;
        item.set_field_guid(ItemFields::Owner as usize, self.get_guid())?;

        if slot < EQUIPMENT_SLOTS_END {
            self.set_field_u32(PlayerFields::PlayerVisibleItem_1Entryid as usize + (slot as usize * 2), item.id)?;
            //todo here: enchants
        }

        Ok(())
    }
}
