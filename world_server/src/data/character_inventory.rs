use crate::{
    prelude::*,
    world::prelude::inventory::{get_compatible_equipment_slots_for_inventory_type, EquipmentSlot, BAG_SLOTS_END, self},
    item::Item,
};
use std::{collections::HashMap, fmt::Display};
use wow_world_base::wrath::ItemSlot;
use wow_world_messages::wrath::{InventoryType, VisibleItem, VisibleItemIndex};

//An identifier for the player inventory (the thing ItemSlot models a cell of)
pub const INVENTORY_SLOT_BAG_0 :u8 = 255;
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

    pub fn take_item(&mut self, slot: EquipmentSlot) -> Option<ItemType> {
        self.items.remove(&slot)
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

impl crate::character::Character
{
    //This function is meant to be used both with inventory and equipment or bags
    //It sets the item in the slot, and returns the old item if there was one
    //Doesn't check if the item is compatible with the slot
    //Slot is u16 because lower 8 bits contain slot data and upper 8 bits contain bag data
    pub fn set_item(&mut self,item : Option<Item>, item_position: (u8,u8) ) -> Result<Option<Item>>
    {
        //let (slot ,bag) = item_position;
        match item_position
        {
            (slot,INVENTORY_SLOT_BAG_0) =>
            {
                if let Ok(equipment_slot) = EquipmentSlot::try_from(slot)
                {
                    let previous_item = self.items.take_item(equipment_slot);
                    if let Some(item) = item
                    {
                        if (slot as u8) <= inventory::EQUIPMENT_SLOTS_END
                        {
                            //TODO: add enchants
                            self.gameplay_data.set_player_visible_item(
                                                                        VisibleItem::new(item.update_state.object_entry().unwrap() as u32,[0u16;2]),
                                                                        VisibleItemIndex::try_from(slot as u8).unwrap()
                                                                    );
                        }
                        self.gameplay_data.set_player_field_inv(ItemSlot::try_from(slot as u8).unwrap(),item.update_state.object_guid().unwrap());
                        self.items.try_insert_item(item)?;
                    }
                    Ok(previous_item)
                }
                else if let Ok(_bag_slot) = inventory::BagSlot::try_from(slot)
                {
                    todo!("Inventory bag not implemented yet")
                }
                else
                {
                    todo!("Non-equipment inventory not implemented yet")
                }

            }
            (_,_) => todo!("Bags not implemented yet")
        }
    }
}