use wow_world_messages::wrath::InventoryType;

pub const EQUIPMENT_SLOTS_START: u8 = 0;
pub const EQUIPMENT_SLOTS_END: u8 = 18;
pub const _BAG_SLOTS_START: u8 = 19;
pub const BAG_SLOTS_END: u8 = 22;
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum EquipmentSlot {
    Head = 0,
    Neck = 1,
    Shoulders = 2,
    Shirt = 3,
    Chest = 4,
    Waist = 5,
    Legs = 6,
    Feet = 7,
    Wrist = 8,
    Hands = 9,
    Finger1 = 10,
    Finger2 = 11,
    Trinket1 = 12,
    Trinket2 = 13,
    Back = 14,
    MainHand = 15,
    Offhand = 16,
    Ranged = 17,
    Tabard = 18,
    Bag1 = 19,
    Bag2 = 20,
    Bag3 = 21,
    Bag4 = 22,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum BagSlot {
    Item1 = 23,
    Item2 = 24,
    Item3 = 25,
    Item4 = 26,
    Item5 = 27,
    Item6 = 28,
    Item7 = 29,
    Item8 = 30,
    Item9 = 31,
    Item10 = 32,
    Item11 = 33,
    Item12 = 34,
    Item13 = 35,
    Item14 = 36,
    Item15 = 37,
    Item16 = 38,
}

pub const fn get_compatible_equipment_slots_for_inventory_type(inventory_type: &InventoryType) -> &[EquipmentSlot] {
    match inventory_type {
        InventoryType::NonEquip => &[],
        InventoryType::Head => &[EquipmentSlot::Head],
        InventoryType::Neck => &[EquipmentSlot::Neck],
        InventoryType::Shoulders => &[EquipmentSlot::Shoulders],
        InventoryType::Body => &[EquipmentSlot::Shirt],
        InventoryType::Chest => &[EquipmentSlot::Chest],
        InventoryType::Waist => &[EquipmentSlot::Waist],
        InventoryType::Legs => &[EquipmentSlot::Legs],
        InventoryType::Feet => &[EquipmentSlot::Feet],
        InventoryType::Wrists => &[EquipmentSlot::Wrist],
        InventoryType::Hands => &[EquipmentSlot::Hands],
        InventoryType::Finger => &[EquipmentSlot::Finger1, EquipmentSlot::Finger2],
        InventoryType::Trinket => &[EquipmentSlot::Trinket1, EquipmentSlot::Trinket2],
        InventoryType::Weapon => &[EquipmentSlot::MainHand],
        InventoryType::Shield => &[EquipmentSlot::Offhand],
        InventoryType::Ranged => &[EquipmentSlot::Ranged],
        InventoryType::Cloak => &[EquipmentSlot::Back],
        InventoryType::TwoHandedWeapon => &[EquipmentSlot::MainHand],
        InventoryType::Bag => &[EquipmentSlot::Bag1, EquipmentSlot::Bag2, EquipmentSlot::Bag3, EquipmentSlot::Bag4],
        InventoryType::Tabard => &[EquipmentSlot::Tabard],
        InventoryType::Robe => &[EquipmentSlot::Chest],
        InventoryType::WeaponMainHand => &[EquipmentSlot::MainHand],
        InventoryType::WeaponOffHand => &[EquipmentSlot::Offhand],
        InventoryType::Holdable => &[EquipmentSlot::Offhand],
        InventoryType::Ammo => &[],
        InventoryType::Thrown => &[EquipmentSlot::Ranged],
        InventoryType::RangedRight => &[EquipmentSlot::Ranged],
        InventoryType::Relic => &[EquipmentSlot::Ranged],
        _ => unimplemented!(),
    }
}

impl TryFrom<u8> for EquipmentSlot {
    type Error = wow_world_messages::errors::EnumError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Head),
            1 => Ok(Self::Neck),
            2 => Ok(Self::Shoulders),
            3 => Ok(Self::Shirt),
            4 => Ok(Self::Chest),
            5 => Ok(Self::Waist),
            6 => Ok(Self::Legs),
            7 => Ok(Self::Feet),
            8 => Ok(Self::Wrist),
            9 => Ok(Self::Hands),
            10 => Ok(Self::Finger1),
            11 => Ok(Self::Finger2),
            12 => Ok(Self::Trinket1),
            13 => Ok(Self::Trinket2),
            14 => Ok(Self::Back),
            15 => Ok(Self::MainHand),
            16 => Ok(Self::Offhand),
            17 => Ok(Self::Ranged),
            18 => Ok(Self::Tabard),
            19 => Ok(Self::Bag1),
            20 => Ok(Self::Bag2),
            21 => Ok(Self::Bag3),
            22 => Ok(Self::Bag4),
            v => Err(wow_world_messages::errors::EnumError::new("EquipmentSlot", v.into())),
        }
    }
}

impl TryFrom<u8> for BagSlot {
    type Error = wow_world_messages::errors::EnumError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            23 => Ok(Self::Item1),
            24 => Ok(Self::Item2),
            25 => Ok(Self::Item3),
            26 => Ok(Self::Item4),
            27 => Ok(Self::Item5),
            28 => Ok(Self::Item6),
            29 => Ok(Self::Item7),
            30 => Ok(Self::Item8),
            31 => Ok(Self::Item9),
            32 => Ok(Self::Item10),
            33 => Ok(Self::Item11),
            34 => Ok(Self::Item12),
            35 => Ok(Self::Item13),
            36 => Ok(Self::Item14),
            37 => Ok(Self::Item15),
            38 => Ok(Self::Item16),
            v => Err(wow_world_messages::errors::EnumError::new("BagSlot", v.into())),
        }
    }
}

impl From<BagSlot> for usize {
    fn from(value: BagSlot) -> Self {
        value as usize
    }
}
