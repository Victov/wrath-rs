use crate::{item::Item, prelude::*};

use super::value_fields::{HasValueFields, ItemFields};

pub trait ItemValueHelpers: HasValueFields {
    fn set_stack_count(&mut self, count: u8) -> Result<()> {
        self.set_item_field_u32(ItemFields::StackCount, count as u32)
    }

    fn set_owner(&mut self, owner_guid: &Guid) -> Result<()> {
        self.set_field_guid(ItemFields::Owner as usize, owner_guid)
    }

    fn set_contained(&mut self, guid: &Guid) -> Result<()> {
        self.set_field_guid(ItemFields::Contained as usize, guid)
    }

    fn set_durability(&mut self, durability: u32) -> Result<()> {
        self.set_field_u32(ItemFields::Durability as usize, durability)
    }

    fn set_max_durability(&mut self, max_durability: u32) -> Result<()> {
        self.set_field_u32(ItemFields::Maxdurability as usize, max_durability)
    }

    fn set_duration(&mut self, duration: u32) -> Result<()> {
        self.set_field_u32(ItemFields::Duration as usize, duration)
    }
}

impl ItemValueHelpers for Item {}
