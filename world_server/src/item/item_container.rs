use std::ops::{Index, IndexMut};

use wow_world_messages::wrath::UpdateItem;

use super::Item;

//generic, index based item container, could represent a bag, bank, loot,
//everything that doesn't require more logic to item placement
pub trait ItemContainer<T>: Index<T, Output = Option<Item>> + IndexMut<T, Output = Option<Item>> + Default
where
    T: Into<usize>,
{
    fn get_items_update_state(&self) -> Vec<UpdateItem>;
}
