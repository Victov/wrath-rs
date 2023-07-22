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