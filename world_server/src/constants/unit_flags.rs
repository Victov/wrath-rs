#[allow(dead_code)]
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

#[allow(dead_code)]
pub enum UnitFlagIndex {
    Unk0 = 0,
    NonAttackable = 1,
    DisableMove = 2,
    PvpAttackable = 3,
    Rename = 4,
    Preparation = 5,
    Unk6 = 6,
    NotAttackable1 = 7,
    OocNotAttackable = 8,
    Passive = 9,
    Looting = 10,
    PetInCombat = 11,
    Pvp = 12,
    Silenced = 13,
    Unk14 = 14,
    Unk15 = 15,
    Unk16 = 16,
    Pacified = 17,
    Stunned = 18,
    InCombat = 19,
    TaxiFlight = 20,
    Disarmed = 21,
    Confused = 22,
    Fleeing = 23,
    PlayerControlled = 24,
    NotSelectable = 25,
    Skinnable = 26,
    Mount = 27,
    Unk28 = 28,
    Unk29 = 29,
    Sheathe = 30,
    //Unk31 = 0x80000000,
}

#[test]
fn test_unit_flags_indices() {
    assert_eq!(1 << (UnitFlagIndex::Unk0 as usize), UnitFlags::Unk0 as usize);
    assert_eq!(1 << (UnitFlagIndex::NonAttackable as usize), UnitFlags::NonAttackable as usize);
    assert_eq!(1 << (UnitFlagIndex::DisableMove as usize), UnitFlags::DisableMove as usize);
    assert_eq!(1 << (UnitFlagIndex::PvpAttackable as usize), UnitFlags::PvpAttackable as usize);
    assert_eq!(1 << (UnitFlagIndex::Rename as usize), UnitFlags::Rename as usize);
    assert_eq!(1 << (UnitFlagIndex::Preparation as usize), UnitFlags::Preparation as usize);
    assert_eq!(1 << (UnitFlagIndex::Unk6 as usize), UnitFlags::Unk6 as usize);
    assert_eq!(1 << (UnitFlagIndex::NotAttackable1 as usize), UnitFlags::NotAttackable1 as usize);
    assert_eq!(1 << (UnitFlagIndex::OocNotAttackable as usize), UnitFlags::OocNotAttackable as usize);
    assert_eq!(1 << (UnitFlagIndex::Passive as usize), UnitFlags::Passive as usize);
    assert_eq!(1 << (UnitFlagIndex::Looting as usize), UnitFlags::Looting as usize);
    assert_eq!(1 << (UnitFlagIndex::PetInCombat as usize), UnitFlags::PetInCombat as usize);
    assert_eq!(1 << (UnitFlagIndex::Pvp as usize), UnitFlags::Pvp as usize);
    assert_eq!(1 << (UnitFlagIndex::Silenced as usize), UnitFlags::Silenced as usize);
    assert_eq!(1 << (UnitFlagIndex::Unk14 as usize), UnitFlags::Unk14 as usize);
    assert_eq!(1 << (UnitFlagIndex::Unk15 as usize), UnitFlags::Unk15 as usize);
    assert_eq!(1 << (UnitFlagIndex::Unk16 as usize), UnitFlags::Unk16 as usize);
    assert_eq!(1 << (UnitFlagIndex::Pacified as usize), UnitFlags::Pacified as usize);
    assert_eq!(1 << (UnitFlagIndex::Stunned as usize), UnitFlags::Stunned as usize);
    assert_eq!(1 << (UnitFlagIndex::InCombat as usize), UnitFlags::InCombat as usize);
    assert_eq!(1 << (UnitFlagIndex::TaxiFlight as usize), UnitFlags::TaxiFlight as usize);
    assert_eq!(1 << (UnitFlagIndex::Disarmed as usize), UnitFlags::Disarmed as usize);
    assert_eq!(1 << (UnitFlagIndex::Confused as usize), UnitFlags::Confused as usize);
    assert_eq!(1 << (UnitFlagIndex::Fleeing as usize), UnitFlags::Fleeing as usize);
    assert_eq!(1 << (UnitFlagIndex::PlayerControlled as usize), UnitFlags::PlayerControlled as usize);
    assert_eq!(1 << (UnitFlagIndex::NotSelectable as usize), UnitFlags::NotSelectable as usize);
    assert_eq!(1 << (UnitFlagIndex::Skinnable as usize), UnitFlags::Skinnable as usize);
    assert_eq!(1 << (UnitFlagIndex::Mount as usize), UnitFlags::Mount as usize);
    assert_eq!(1 << (UnitFlagIndex::Unk28 as usize), UnitFlags::Unk28 as usize);
    assert_eq!(1 << (UnitFlagIndex::Unk29 as usize), UnitFlags::Unk29 as usize);
    assert_eq!(1 << (UnitFlagIndex::Sheathe as usize), UnitFlags::Sheathe as usize);
    //Unk31 = 0x80000000,
}
