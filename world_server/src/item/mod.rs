mod item_database;

use wow_world_messages::{wrath::{UpdateItem, UpdateItemBuilder}, Guid};
use wow_items::wrath::lookup_item;
use wrath_realm_db::item_instance::DBItemInstance;

use crate::data::InventoryStorable;

#[derive(Default)]
pub struct Item{
   pub update_state :UpdateItem
}

impl Item{
}
impl std::fmt::Display for Item
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         write!(f,"Item")
    }
}
impl InventoryStorable for Item{
    fn get_inventory_type(&self) -> wow_world_base::wrath::InventoryType {
    //TODO: this might need to change when templates are loaded from DB
      lookup_item(self.update_state.object_entry().unwrap().try_into().unwrap()).unwrap().inventory_type()
    }
}

impl From<&DBItemInstance> for Item
   {
       fn from(value: &DBItemInstance) -> Self {
         //TODO: this object guid is now good, but  there is no id in character_equipment table
         Item{
             update_state: UpdateItemBuilder::new()
             .set_object_guid(((value.character_id as u64) + value.slot_id as u64).into())
             .set_object_entry(value.item.try_into().unwrap())
             .set_object_scale_x(1.0)
             .set_item_owner(Guid::new(value.character_id as u64))
             .set_item_contained(Guid::new(value.character_id as u64))
             .set_item_stack_count(1)
             .set_item_durability(100)
             .set_item_maxdurability(100)
             .finalize()
         }
       }
   }