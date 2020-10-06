#[allow(dead_code)]
pub enum ObjectUpdateType
{
    Values = 0,
    Movement = 1,
    CreateObject = 2,
    CreateObject2 = 3,
    OutOfRangeObjects = 4,
    NearObjects = 5,
}

#[allow(dead_code)]
pub enum ObjectUpdateFlags
{
    FlagNone = 0,
    FlagSelf =  0x0001,
    Transport = 0x0002,
    HasTarget = 0x0004,
    Unknown = 0x0008,
    LowGuid = 0x0010,
    Living = 0x0020,
    StationaryPosition = 0x0040,
    Vehicle = 0x0080,
    Position = 0x0100,
    Rotation = 0x0200,
}

#[allow(dead_code)]
pub enum ObjectType
{
    Object,
    Item,
    Container,
    Unit,
    Player,
    GameObject,
    DynamicObject,
    Corpse,
}

