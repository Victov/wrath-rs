pub enum UnitFlags {
    Unk0 = 0x00000001,
    NonAttackable = 0x00000002,
    DisableMove = 0x00000004,
    PvpAttackable = 0x00000008,
    Rename = 0x00000010,
    Preparation = 0x00000020,
    Unk6 = 0x00000040,
    NotAttackable1 = 0x00000080,
    OocNotAttackable = 0x00000100,
    Passive = 0x00000200,
    Looting = 0x00000400,
    PetInCombat = 0x00000800,
    Pvp = 0x00001000,
    Silenced = 0x00002000,
    Unk14 = 0x00004000,
    Unk15 = 0x00008000,
    Unk16 = 0x00010000,
    Pacified = 0x00020000,
    Stunned = 0x00040000,
    InCombat = 0x00080000,
    TaxiFlight = 0x00100000,
    Disarmed = 0x00200000,
    Confused = 0x00400000,
    Fleeing = 0x00800000,
    PlayerControlled = 0x01000000,
    NotSelectable = 0x02000000,
    Skinnable = 0x04000000,
    Mount = 0x08000000,
    Unk28 = 0x10000000,
    Unk29 = 0x20000000,
    Sheathe = 0x40000000,
    //Unk31 = 0x80000000,
}