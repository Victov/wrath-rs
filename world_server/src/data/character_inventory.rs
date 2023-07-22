use crate::{
    prelude::*,
    world::prelude::inventory::{get_compatible_equipment_slots_for_inventory_type, EquipmentSlot, BAG_SLOTS_END},
    item::Item,
};
use std::{collections::HashMap, fmt::Display};
use wow_world_messages::wrath::InventoryType;

//Models anything that can be stored in the inventory
pub trait InventoryStorable: Display {
    fn get_inventory_type(&self) -> InventoryType;
}

// Struct that can be used to model the character equipment slots and their fillings
// Generic so that it can be used in different cases (simple case during character creation, or
// full feature version during gameplay)
pub struct CharacterInventory<ItemType: InventoryStorable> {
    items: HashMap<EquipmentSlot, ItemType>,
}

impl<ItemType: InventoryStorable> CharacterInventory<ItemType> {
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }

    //Tries to insert item, returns Ok(inserted slot) if successful
    pub fn try_insert_item(&mut self, item: ItemType) -> Result<EquipmentSlot> {
        let inventory_type = item.get_inventory_type();
        let possible_slots = get_compatible_equipment_slots_for_inventory_type(&inventory_type);

        for &possible_slot in possible_slots {
            if self.items.get(&possible_slot).is_none() {
                self.items.insert(possible_slot, item);
                return Ok(possible_slot);
            }
        }
        bail!("No free slots to put item {}", item);
    }

    pub fn get_item(&self, slot: EquipmentSlot) -> Option<&ItemType> {
        self.items.get(&slot)
    }
    pub fn get_all_equipment(&self) -> [Option<&ItemType>;(BAG_SLOTS_END + 1) as usize]
    {
        let mut result = [None; (BAG_SLOTS_END + 1) as usize];
        for (slot, item) in self.items.iter()
        {
            result[*slot as usize] = Some(item);
        }
        result
    }
}

#[derive(Copy, Clone)]
pub struct SimpleItemDescription {
    pub item_id: u32,
    pub inventory_type: InventoryType,
}

impl Display for SimpleItemDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SimpleItemDescription {{ item_id: {}, inventory_type: {} }}",
            self.item_id, self.inventory_type
        )
    }
}

impl InventoryStorable for SimpleItemDescription {
    fn get_inventory_type(&self) -> InventoryType {
        self.inventory_type
    }
}

pub type SimpleCharacterInventory = CharacterInventory<SimpleItemDescription>;
pub type GameplayCharacterInventory = CharacterInventory<Item>;