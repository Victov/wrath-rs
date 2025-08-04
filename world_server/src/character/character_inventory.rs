use crate::item::item_container::ItemContainer;
use crate::{
    item::Item,
    prelude::*,
    world::prelude::inventory::{self, get_compatible_equipment_slots_for_inventory_type, BagSlot, EquipmentSlot, BAG_SLOTS_END},
};
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Index, IndexMut},
};
use wow_world_base::wrath::ItemSlot;
use wow_world_messages::wrath::UpdateItem;
use wow_world_messages::wrath::{InventoryType, VisibleItem, VisibleItemIndex};

//An identifier for the player inventory (the thing ItemSlot models a cell of)
pub const INVENTORY_SLOT_BAG_0: u8 = 255;
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

impl<ItemType: InventoryStorable> Default for CharacterInventory<ItemType> {
    fn default() -> Self {
        Self::new()
    }
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
            if let std::collections::hash_map::Entry::Vacant(e) = self.items.entry(possible_slot) {
                e.insert(item);
                return Ok(possible_slot);
            }
        }
        bail!("No free slots to put item {}", item);
    }

    pub fn get_item(&self, slot: EquipmentSlot) -> Option<&ItemType> {
        self.items.get(&slot)
    }

    pub fn take_item(&mut self, slot: EquipmentSlot) -> Option<ItemType> {
        self.items.remove(&slot)
    }

    pub fn get_all_equipment(&self) -> [Option<&ItemType>; (BAG_SLOTS_END + 1) as usize] {
        let mut result = [None; (BAG_SLOTS_END + 1) as usize];
        for (slot, item) in self.items.iter() {
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

#[derive(Default)]
pub struct BagInventory {
    items: [Option<Item>; 16],
}
impl Index<BagSlot> for BagInventory {
    type Output = Option<Item>;

    fn index(&self, index: BagSlot) -> &Self::Output {
        &self.items[Self::to_index(&index)]
    }
}
impl IndexMut<BagSlot> for BagInventory {
    fn index_mut(&mut self, index: BagSlot) -> &mut Self::Output {
        &mut self.items[Self::to_index(&index)]
    }
}

impl BagInventory {
    fn to_index(slot: &BagSlot) -> usize {
        (*slot as usize) - (BagSlot::Item1 as usize)
    }
    fn take_item(&mut self, index: BagSlot) -> Option<Item> {
        let index = Self::to_index(&index);
        if self.items[index].is_some() {
            self.items[index].take()
        } else {
            None
        }
    }
}

impl ItemContainer<BagSlot> for BagInventory {
    fn get_items_update_state(&self) -> Vec<UpdateItem> {
        let mut updates = Vec::new();
        for item in self.items.iter().flatten() {
            updates.push(item.update_state.clone());
        }
        updates
    }
}

impl crate::character::Character {
    //This function is meant to be used both with inventory and equipment or bags
    //It sets the item in the slot, and returns the old item if there was one
    //Doesn't check if the item is compatible with the slot
    //Slot is u16 because lower 8 bits contain slot data and upper 8 bits contain bag data
    pub fn set_item(&mut self, item: Option<Item>, item_position: (u8, u8)) -> Result<Option<Item>> {
        match item_position {
            (slot, INVENTORY_SLOT_BAG_0) => {
                if let Ok(equipment_slot) = EquipmentSlot::try_from(slot) {
                    let previous_item = self.equipped_items.take_item(equipment_slot);
                    if let Some(item) = item {
                        if slot <= inventory::EQUIPMENT_SLOTS_END {
                            //TODO: add display enchants
                            self.gameplay_data.set_player_visible_item(
                                VisibleItem::new(item.update_state.object_entry().unwrap() as u32, [0u16; 2]),
                                VisibleItemIndex::try_from(slot).unwrap(),
                            );
                        }
                        self.gameplay_data
                            .set_player_field_inv(ItemSlot::try_from(slot).unwrap(), item.update_state.object_guid().unwrap());
                        self.equipped_items.try_insert_item(item)?;
                    } else {
                        if slot <= inventory::EQUIPMENT_SLOTS_END {
                            self.gameplay_data
                                .set_player_visible_item(VisibleItem::new(0u32, [0u16; 2]), VisibleItemIndex::try_from(slot).unwrap());
                        }
                        self.gameplay_data.set_player_field_inv(ItemSlot::try_from(slot).unwrap(), Guid::zero());
                    }
                    Ok(previous_item)
                } else if let Ok(bag_slot) = inventory::BagSlot::try_from(slot) {
                    //TODO: add persistent bag for character and complete the implementation here
                    let previous_item = self.bag_items.take_item(bag_slot);
                    if let Some(item) = item {
                        self.gameplay_data
                            .set_player_field_inv(ItemSlot::try_from(slot).unwrap(), item.update_state.object_guid().unwrap());
                        self.bag_items[bag_slot] = Some(item);
                    } else {
                        self.gameplay_data.set_player_field_inv(ItemSlot::try_from(slot).unwrap(), Guid::zero());
                        self.bag_items[bag_slot] = None;
                    }
                    Ok(previous_item)
                } else {
                    todo!("Non-equipment inventory not implemented yet")
                }
            }
            (_, _) => todo!("Bags not implemented yet"),
        }
    }

    //Attempt to auto-equip (right click equipable item from inventory) an item.
    //Returns the item that was prevously in that equipment slot, or None if the slot was empty.
    pub fn auto_equip_item_from_bag(&mut self, item_position: (u8, u8)) -> Result<Option<Item>> {
        match item_position {
            (slot, INVENTORY_SLOT_BAG_0) => {
                if let Ok(bag_slot) = inventory::BagSlot::try_from(slot) {
                    let item_to_equip = self.bag_items.take_item(bag_slot);
                    if let Some(item) = item_to_equip {
                        let item_inventory_type = item.get_inventory_type();
                        let possible_equip_slots = get_compatible_equipment_slots_for_inventory_type(&item_inventory_type);

                        let mut best_slot_candidate = None;
                        for possible_equip_slot in possible_equip_slots {
                            if best_slot_candidate.is_none() {
                                best_slot_candidate = Some(possible_equip_slot);
                            }
                            if self.equipped_items.get_item(*possible_equip_slot).is_none() {
                                //If we find a slot that has no other items inside, we have landed
                                //on a winner automatically. We can stop.
                                best_slot_candidate = Some(possible_equip_slot);
                                break;
                            }
                        }

                        if let Some(picked_slot) = best_slot_candidate {
                            //Replace the item
                            let previous_item_in_slot = self.set_item(Some(item), (*picked_slot as u8, INVENTORY_SLOT_BAG_0))?;
                            return Ok(previous_item_in_slot);
                        } else {
                            warn!("No slot found to auto-equip this item into!");
                        }
                    } else {
                        warn!("Attempting to auto-equip a non-existant item");
                    }
                }
            }
            (_, _) => todo!("Bags not implemented yet"),
        }
        bail!("Item not from bag 1");
    }

    #[allow(dead_code)]
    fn has_item_in_slot(&self, item_position: (u8, u8)) -> bool {
        match item_position {
            (slot, INVENTORY_SLOT_BAG_0) => {
                if let Ok(equipment_slot) = EquipmentSlot::try_from(slot) {
                    self.equipped_items.get_item(equipment_slot).is_some()
                } else if let Ok(bag_slot) = inventory::BagSlot::try_from(slot) {
                    self.bag_items[bag_slot].is_some()
                } else {
                    todo!("Non-equipment inventory not implemented yet")
                }
            }
            (_, _) => todo!("Bags not implemented yet"),
        }
    }
}
