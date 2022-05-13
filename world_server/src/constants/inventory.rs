#[allow(dead_code)]
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
}

pub const EQUIPMENT_SLOTS_START: u8 = 0;
pub const EQUIPMENT_SLOTS_END: u8 = 18;
pub const BAG_SLOTS_START: u8 = 19;
pub const BAG_SLOTS_END: u8 = 22;

#[allow(dead_code)]
pub enum InventorySlot {
    Bag1 = 19,
    Bag2 = 20,
    Bag3 = 21,
    Bag4 = 22,
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
