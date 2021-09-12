#[allow(dead_code)]
#[repr(u8)]
pub enum ObjectUpdateType {
    Values = 0,
    Movement = 1,
    CreateObject = 2,
    CreateYourself = 3,
    OutOfRangeObjects = 4,
}

#[allow(dead_code)]
pub enum ObjectUpdateFlags {
    None = 0x00,
    UpdateSelf = 0x01,
    Transport = 0x02,
    HasTarget = 0x04,
    LowGuid = 0x08,
    HighGuid = 0x10,
    Living = 0x20,
    HasPosition = 0x40,
    Vehicle = 0x80,
    Position = 0x100,
    Rotation = 0x200,
}

#[allow(dead_code)]
pub enum ObjectType {
    Object = 0,
    Item = 1,
    Container = 2,
    Unit = 3,
    Player = 4,
    GameObject = 5,
    DynamicObject = 6,
    Corpse = 7,
    AIGroup = 8,
    AreaTrigger = 9,
}
