//From https://github.com/arcemu/arcemu/blob/master/src/world/Game/Entities/Update/UpdateFields.h
// Auto generated for version 3, 3, 5, 12340, regexed into rust code with pain

#[allow(dead_code)]
pub enum ObjectFields {
    LowGuid = 0x0000,  // Size: 2, Type: Long, Flags: Public
    HighGuid = 0x0001, //Not auto-generated, inserted that myself
    Type = 0x0002,     // Size: 1, Type: Int, Flags: Public
    Entry = 0x0003,    // Size: 1, Type: Int, Flags: Public
    Scale = 0x0004,    // Size: 1, Type: Float, Flags: Public
    Padding = 0x0005,  // Size: 1, Type: Int, Flags: None
    End = 0x0006,      //(ObjectFields::End as isize) = 0x0006
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum ItemFields {
    Owner = (ObjectFields::End as isize) + 0x0000, // Size: 2, Type: Long, Flags: Public
    Contained = (ObjectFields::End as isize) + 0x0002, // Size: 2, Type: Long, Flags: Public
    Creator = (ObjectFields::End as isize) + 0x0004, // Size: 2, Type: Long, Flags: Public
    Giftcreator = (ObjectFields::End as isize) + 0x0006, // Size: 2, Type: Long, Flags: Public
    StackCount = (ObjectFields::End as isize) + 0x0008, // Size: 1, Type: Int, Flags: Owner, ItemOwner
    Duration = (ObjectFields::End as isize) + 0x0009, // Size: 1, Type: Int, Flags: Owner, ItemOwner
    SpellCharges = (ObjectFields::End as isize) + 0x000A, // Size: 5, Type: Int, Flags: Owner, ItemOwner
    Flags = (ObjectFields::End as isize) + 0x000F,        // Size: 1, Type: Int, Flags: Public
    Enchantment_1_1 = (ObjectFields::End as isize) + 0x0010, // Size: 2, Type: Int, Flags: Public
    Enchantment_1_3 = (ObjectFields::End as isize) + 0x0012, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_2_1 = (ObjectFields::End as isize) + 0x0013, // Size: 2, Type: Int, Flags: Public
    Enchantment_2_3 = (ObjectFields::End as isize) + 0x0015, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_3_1 = (ObjectFields::End as isize) + 0x0016, // Size: 2, Type: Int, Flags: Public
    Enchantment_3_3 = (ObjectFields::End as isize) + 0x0018, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_4_1 = (ObjectFields::End as isize) + 0x0019, // Size: 2, Type: Int, Flags: Public
    Enchantment_4_3 = (ObjectFields::End as isize) + 0x001B, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_5_1 = (ObjectFields::End as isize) + 0x001C, // Size: 2, Type: Int, Flags: Public
    Enchantment_5_3 = (ObjectFields::End as isize) + 0x001E, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_6_1 = (ObjectFields::End as isize) + 0x001F, // Size: 2, Type: Int, Flags: Public
    Enchantment_6_3 = (ObjectFields::End as isize) + 0x0021, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_7_1 = (ObjectFields::End as isize) + 0x0022, // Size: 2, Type: Int, Flags: Public
    Enchantment_7_3 = (ObjectFields::End as isize) + 0x0024, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_8_1 = (ObjectFields::End as isize) + 0x0025, // Size: 2, Type: Int, Flags: Public
    Enchantment_8_3 = (ObjectFields::End as isize) + 0x0027, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_9_1 = (ObjectFields::End as isize) + 0x0028, // Size: 2, Type: Int, Flags: Public
    Enchantment_9_3 = (ObjectFields::End as isize) + 0x002A, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_10_1 = (ObjectFields::End as isize) + 0x002B, // Size: 2, Type: Int, Flags: Public
    Enchantment_10_3 = (ObjectFields::End as isize) + 0x002D, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_11_1 = (ObjectFields::End as isize) + 0x002E, // Size: 2, Type: Int, Flags: Public
    Enchantment_11_3 = (ObjectFields::End as isize) + 0x0030, // Size: 1, Type: TwoShort, Flags: Public
    Enchantment_12_1 = (ObjectFields::End as isize) + 0x0031, // Size: 2, Type: Int, Flags: Public
    Enchantment_12_3 = (ObjectFields::End as isize) + 0x0033, // Size: 1, Type: TwoShort, Flags: Public
    PropertySeed = (ObjectFields::End as isize) + 0x0034,     // Size: 1, Type: Int, Flags: Public
    RandomPropertiesId = (ObjectFields::End as isize) + 0x0035, // Size: 1, Type: Int, Flags: Public
    Durability = (ObjectFields::End as isize) + 0x0036, // Size: 1, Type: Int, Flags: Owner, ItemOwner
    Maxdurability = (ObjectFields::End as isize) + 0x0037, // Size: 1, Type: Int, Flags: Owner, ItemOwner
    CreatePlayedTime = (ObjectFields::End as isize) + 0x0038, // Size: 1, Type: Int, Flags: Public
    Pad = (ObjectFields::End as isize) + 0x0039,           // Size: 1, Type: Int, Flags: None
    End = (ObjectFields::End as isize) + 0x003A,
}

#[allow(dead_code)]
pub enum ContainerFields {
    NumSlots = (ItemFields::End as isize) + 0x0000, // Size: 1, Type: Int, Flags: Public
    ContainerAlignPad = (ItemFields::End as isize) + 0x0001, // Size: 1, Type: Bytes, Flags: None
    Slot1 = (ItemFields::End as isize) + 0x0002,    // Size: 72, Type: Long, Flags: Public
    ContainerEnd = (ItemFields::End as isize) + 0x004A,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum UnitFields {
    Charm = (ObjectFields::End as isize) + 0x0000, // Size: 2, Type: Long, Flags: Public
    Summon = (ObjectFields::End as isize) + 0x0002, // Size: 2, Type: Long, Flags: Public
    Critter = (ObjectFields::End as isize) + 0x0004, // Size: 2, Type: Long, Flags: Private
    Charmedby = (ObjectFields::End as isize) + 0x0006, // Size: 2, Type: Long, Flags: Public
    Summonedby = (ObjectFields::End as isize) + 0x0008, // Size: 2, Type: Long, Flags: Public
    Createdby = (ObjectFields::End as isize) + 0x000A, // Size: 2, Type: Long, Flags: Public
    Target = (ObjectFields::End as isize) + 0x000C, // Size: 2, Type: Long, Flags: Public
    ChannelObject = (ObjectFields::End as isize) + 0x000E, // Size: 2, Type: Long, Flags: Public
    UnitChannelSpell = (ObjectFields::End as isize) + 0x0010, // Size: 1, Type: Int, Flags: Public
    UnitBytes0 = (ObjectFields::End as isize) + 0x0011, // Size: 1, Type: Bytes, Flags: Public
    Health = (ObjectFields::End as isize) + 0x0012, // Size: 1, Type: Int, Flags: Public
    Power1 = (ObjectFields::End as isize) + 0x0013, // Size: 1, Type: Int, Flags: Public
    Power2 = (ObjectFields::End as isize) + 0x0014, // Size: 1, Type: Int, Flags: Public
    Power3 = (ObjectFields::End as isize) + 0x0015, // Size: 1, Type: Int, Flags: Public
    Power4 = (ObjectFields::End as isize) + 0x0016, // Size: 1, Type: Int, Flags: Public
    Power5 = (ObjectFields::End as isize) + 0x0017, // Size: 1, Type: Int, Flags: Public
    Power6 = (ObjectFields::End as isize) + 0x0018, // Size: 1, Type: Int, Flags: Public
    Power7 = (ObjectFields::End as isize) + 0x0019, // Size: 1, Type: Int, Flags: Public
    Maxhealth = (ObjectFields::End as isize) + 0x001A, // Size: 1, Type: Int, Flags: Public
    Maxpower1 = (ObjectFields::End as isize) + 0x001B, // Size: 1, Type: Int, Flags: Public
    Maxpower2 = (ObjectFields::End as isize) + 0x001C, // Size: 1, Type: Int, Flags: Public
    Maxpower3 = (ObjectFields::End as isize) + 0x001D, // Size: 1, Type: Int, Flags: Public
    Maxpower4 = (ObjectFields::End as isize) + 0x001E, // Size: 1, Type: Int, Flags: Public
    Maxpower5 = (ObjectFields::End as isize) + 0x001F, // Size: 1, Type: Int, Flags: Public
    Maxpower6 = (ObjectFields::End as isize) + 0x0020, // Size: 1, Type: Int, Flags: Public
    Maxpower7 = (ObjectFields::End as isize) + 0x0021, // Size: 1, Type: Int, Flags: Public
    PowerRegenFlatModifier = (ObjectFields::End as isize) + 0x0022, // Size: 7, Type: Float, Flags: Private, Owner
    PowerRegenInterruptedFlatModifier = (ObjectFields::End as isize) + 0x0029, // Size: 7, Type: Float, Flags: Private, Owner
    Level = (ObjectFields::End as isize) + 0x0030, // Size: 1, Type: Int, Flags: Public
    Factiontemplate = (ObjectFields::End as isize) + 0x0031, // Size: 1, Type: Int, Flags: Public
    UnitVirtualItemSlotId = (ObjectFields::End as isize) + 0x0032, // Size: 3, Type: Int, Flags: Public
    UnitFlags = (ObjectFields::End as isize) + 0x0035, // Size: 1, Type: Int, Flags: Public
    UnitFlags2 = (ObjectFields::End as isize) + 0x0036, // Size: 1, Type: Int, Flags: Public
    Aurastate = (ObjectFields::End as isize) + 0x0037, // Size: 1, Type: Int, Flags: Public
    Baseattacktime = (ObjectFields::End as isize) + 0x0038, // Size: 2, Type: Int, Flags: Public
    Rangedattacktime = (ObjectFields::End as isize) + 0x003A, // Size: 1, Type: Int, Flags: Private
    Boundingradius = (ObjectFields::End as isize) + 0x003B, // Size: 1, Type: Float, Flags: Public
    Combatreach = (ObjectFields::End as isize) + 0x003C, // Size: 1, Type: Float, Flags: Public
    Displayid = (ObjectFields::End as isize) + 0x003D, // Size: 1, Type: Int, Flags: Public
    Nativedisplayid = (ObjectFields::End as isize) + 0x003E, // Size: 1, Type: Int, Flags: Public
    Mountdisplayid = (ObjectFields::End as isize) + 0x003F, // Size: 1, Type: Int, Flags: Public
    Mindamage = (ObjectFields::End as isize) + 0x0040, // Size: 1, Type: Float, Flags: Private, Owner, PartyLeader
    Maxdamage = (ObjectFields::End as isize) + 0x0041, // Size: 1, Type: Float, Flags: Private, Owner, PartyLeader
    Minoffhanddamage = (ObjectFields::End as isize) + 0x0042, // Size: 1, Type: Float, Flags: Private, Owner, PartyLeader
    Maxoffhanddamage = (ObjectFields::End as isize) + 0x0043, // Size: 1, Type: Float, Flags: Private, Owner, PartyLeader
    UnitBytes1 = (ObjectFields::End as isize) + 0x0044,       // Size: 1, Type: Bytes, Flags: Public
    Petnumber = (ObjectFields::End as isize) + 0x0045,        // Size: 1, Type: Int, Flags: Public
    PetNameTimestamp = (ObjectFields::End as isize) + 0x0046, // Size: 1, Type: Int, Flags: Public
    Petexperience = (ObjectFields::End as isize) + 0x0047,    // Size: 1, Type: Int, Flags: Owner
    Petnextlevelexp = (ObjectFields::End as isize) + 0x0048,  // Size: 1, Type: Int, Flags: Owner
    UnitDynamicFlags = (ObjectFields::End as isize) + 0x0049, // Size: 1, Type: Int, Flags: Dynamic
    UnitModCastSpeed = (ObjectFields::End as isize) + 0x004A, // Size: 1, Type: Float, Flags: Public
    UnitCreatedBySpell = (ObjectFields::End as isize) + 0x004B, // Size: 1, Type: Int, Flags: Public
    UnitNpcFlags = (ObjectFields::End as isize) + 0x004C,     // Size: 1, Type: Int, Flags: Dynamic
    UnitNpcEmotestate = (ObjectFields::End as isize) + 0x004D, // Size: 1, Type: Int, Flags: Public
    Stat0 = (ObjectFields::End as isize) + 0x004E, // Size: 1, Type: Int, Flags: Private, Owner
    Stat1 = (ObjectFields::End as isize) + 0x004F, // Size: 1, Type: Int, Flags: Private, Owner
    Stat2 = (ObjectFields::End as isize) + 0x0050, // Size: 1, Type: Int, Flags: Private, Owner
    Stat3 = (ObjectFields::End as isize) + 0x0051, // Size: 1, Type: Int, Flags: Private, Owner
    Stat4 = (ObjectFields::End as isize) + 0x0052, // Size: 1, Type: Int, Flags: Private, Owner
    Posstat0 = (ObjectFields::End as isize) + 0x0053, // Size: 1, Type: Int, Flags: Private, Owner
    Posstat1 = (ObjectFields::End as isize) + 0x0054, // Size: 1, Type: Int, Flags: Private, Owner
    Posstat2 = (ObjectFields::End as isize) + 0x0055, // Size: 1, Type: Int, Flags: Private, Owner
    Posstat3 = (ObjectFields::End as isize) + 0x0056, // Size: 1, Type: Int, Flags: Private, Owner
    Posstat4 = (ObjectFields::End as isize) + 0x0057, // Size: 1, Type: Int, Flags: Private, Owner
    Negstat0 = (ObjectFields::End as isize) + 0x0058, // Size: 1, Type: Int, Flags: Private, Owner
    Negstat1 = (ObjectFields::End as isize) + 0x0059, // Size: 1, Type: Int, Flags: Private, Owner
    Negstat2 = (ObjectFields::End as isize) + 0x005A, // Size: 1, Type: Int, Flags: Private, Owner
    Negstat3 = (ObjectFields::End as isize) + 0x005B, // Size: 1, Type: Int, Flags: Private, Owner
    Negstat4 = (ObjectFields::End as isize) + 0x005C, // Size: 1, Type: Int, Flags: Private, Owner
    Resistances = (ObjectFields::End as isize) + 0x005D, // Size: 7, Type: Int, Flags: Private, Owner, PartyLeader
    Resistancebuffmodspositive = (ObjectFields::End as isize) + 0x0064, // Size: 7, Type: Int, Flags: Private, Owner
    Resistancebuffmodsnegative = (ObjectFields::End as isize) + 0x006B, // Size: 7, Type: Int, Flags: Private, Owner
    BaseMana = (ObjectFields::End as isize) + 0x0072, // Size: 1, Type: Int, Flags: Public
    BaseHealth = (ObjectFields::End as isize) + 0x0073, // Size: 1, Type: Int, Flags: Private, Owner
    UnitBytes2 = (ObjectFields::End as isize) + 0x0074, // Size: 1, Type: Bytes, Flags: Public
    AttackPower = (ObjectFields::End as isize) + 0x0075, // Size: 1, Type: Int, Flags: Private, Owner
    AttackPowerMods = (ObjectFields::End as isize) + 0x0076, // Size: 1, Type: TwoShort, Flags: Private, Owner
    AttackPowerMultiplier = (ObjectFields::End as isize) + 0x0077, // Size: 1, Type: Float, Flags: Private, Owner
    RangedAttackPower = (ObjectFields::End as isize) + 0x0078, // Size: 1, Type: Int, Flags: Private, Owner
    RangedAttackPowerMods = (ObjectFields::End as isize) + 0x0079, // Size: 1, Type: TwoShort, Flags: Private, Owner
    RangedAttackPowerMultiplier = (ObjectFields::End as isize) + 0x007A, // Size: 1, Type: Float, Flags: Private, Owner
    Minrangeddamage = (ObjectFields::End as isize) + 0x007B, // Size: 1, Type: Float, Flags: Private, Owner
    Maxrangeddamage = (ObjectFields::End as isize) + 0x007C, // Size: 1, Type: Float, Flags: Private, Owner
    PowerCostModifier = (ObjectFields::End as isize) + 0x007D, // Size: 7, Type: Int, Flags: Private, Owner
    PowerCostMultiplier = (ObjectFields::End as isize) + 0x0084, // Size: 7, Type: Float, Flags: Private, Owner
    Maxhealthmodifier = (ObjectFields::End as isize) + 0x008B, // Size: 1, Type: Float, Flags: Private, Owner
    Hoverheight = (ObjectFields::End as isize) + 0x008C, // Size: 1, Type: Float, Flags: Public
    Padding = (ObjectFields::End as isize) + 0x008D,     // Size: 1, Type: Int, Flags: None
    UnitEnd = (ObjectFields::End as isize) + 0x008E,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum PlayerFields {
    PlayerDuelArbiter = (UnitFields::UnitEnd as isize) + 0x0000, // Size: 2, Type: Long, Flags: Public
    PlayerFlags = (UnitFields::UnitEnd as isize) + 0x0002, // Size: 1, Type: Int, Flags: Public
    PlayerGuildid = (UnitFields::UnitEnd as isize) + 0x0003, // Size: 1, Type: Int, Flags: Public
    PlayerGuildrank = (UnitFields::UnitEnd as isize) + 0x0004, // Size: 1, Type: Int, Flags: Public
    PlayerBytes = (UnitFields::UnitEnd as isize) + 0x0005, // Size: 1, Type: Bytes, Flags: Public
    PlayerBytes2 = (UnitFields::UnitEnd as isize) + 0x0006, // Size: 1, Type: Bytes, Flags: Public
    PlayerBytes3 = (UnitFields::UnitEnd as isize) + 0x0007, // Size: 1, Type: Bytes, Flags: Public
    PlayerDuelTeam = (UnitFields::UnitEnd as isize) + 0x0008, // Size: 1, Type: Int, Flags: Public
    PlayerGuildTimestamp = (UnitFields::UnitEnd as isize) + 0x0009, // Size: 1, Type: Int, Flags: Public
    PlayerQuestLog_1_1 = (UnitFields::UnitEnd as isize) + 0x000A, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_1_2 = (UnitFields::UnitEnd as isize) + 0x000B, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_1_3 = (UnitFields::UnitEnd as isize) + 0x000C, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_1_4 = (UnitFields::UnitEnd as isize) + 0x000E, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_2_1 = (UnitFields::UnitEnd as isize) + 0x000F, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_2_2 = (UnitFields::UnitEnd as isize) + 0x0010, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_2_3 = (UnitFields::UnitEnd as isize) + 0x0011, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_2_5 = (UnitFields::UnitEnd as isize) + 0x0013, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_3_1 = (UnitFields::UnitEnd as isize) + 0x0014, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_3_2 = (UnitFields::UnitEnd as isize) + 0x0015, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_3_3 = (UnitFields::UnitEnd as isize) + 0x0016, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_3_5 = (UnitFields::UnitEnd as isize) + 0x0018, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_4_1 = (UnitFields::UnitEnd as isize) + 0x0019, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_4_2 = (UnitFields::UnitEnd as isize) + 0x001A, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_4_3 = (UnitFields::UnitEnd as isize) + 0x001B, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_4_5 = (UnitFields::UnitEnd as isize) + 0x001D, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_5_1 = (UnitFields::UnitEnd as isize) + 0x001E, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_5_2 = (UnitFields::UnitEnd as isize) + 0x001F, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_5_3 = (UnitFields::UnitEnd as isize) + 0x0020, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_5_5 = (UnitFields::UnitEnd as isize) + 0x0022, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_6_1 = (UnitFields::UnitEnd as isize) + 0x0023, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_6_2 = (UnitFields::UnitEnd as isize) + 0x0024, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_6_3 = (UnitFields::UnitEnd as isize) + 0x0025, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_6_5 = (UnitFields::UnitEnd as isize) + 0x0027, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_7_1 = (UnitFields::UnitEnd as isize) + 0x0028, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_7_2 = (UnitFields::UnitEnd as isize) + 0x0029, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_7_3 = (UnitFields::UnitEnd as isize) + 0x002A, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_7_5 = (UnitFields::UnitEnd as isize) + 0x002C, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_8_1 = (UnitFields::UnitEnd as isize) + 0x002D, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_8_2 = (UnitFields::UnitEnd as isize) + 0x002E, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_8_3 = (UnitFields::UnitEnd as isize) + 0x002F, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_8_5 = (UnitFields::UnitEnd as isize) + 0x0031, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_9_1 = (UnitFields::UnitEnd as isize) + 0x0032, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_9_2 = (UnitFields::UnitEnd as isize) + 0x0033, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_9_3 = (UnitFields::UnitEnd as isize) + 0x0034, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_9_5 = (UnitFields::UnitEnd as isize) + 0x0036, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_10_1 = (UnitFields::UnitEnd as isize) + 0x0037, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_10_2 = (UnitFields::UnitEnd as isize) + 0x0038, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_10_3 = (UnitFields::UnitEnd as isize) + 0x0039, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_10_5 = (UnitFields::UnitEnd as isize) + 0x003B, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_11_1 = (UnitFields::UnitEnd as isize) + 0x003C, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_11_2 = (UnitFields::UnitEnd as isize) + 0x003D, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_11_3 = (UnitFields::UnitEnd as isize) + 0x003E, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_11_5 = (UnitFields::UnitEnd as isize) + 0x0040, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_12_1 = (UnitFields::UnitEnd as isize) + 0x0041, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_12_2 = (UnitFields::UnitEnd as isize) + 0x0042, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_12_3 = (UnitFields::UnitEnd as isize) + 0x0043, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_12_5 = (UnitFields::UnitEnd as isize) + 0x0045, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_13_1 = (UnitFields::UnitEnd as isize) + 0x0046, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_13_2 = (UnitFields::UnitEnd as isize) + 0x0047, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_13_3 = (UnitFields::UnitEnd as isize) + 0x0048, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_13_5 = (UnitFields::UnitEnd as isize) + 0x004A, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_14_1 = (UnitFields::UnitEnd as isize) + 0x004B, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_14_2 = (UnitFields::UnitEnd as isize) + 0x004C, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_14_3 = (UnitFields::UnitEnd as isize) + 0x004D, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_14_5 = (UnitFields::UnitEnd as isize) + 0x004F, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_15_1 = (UnitFields::UnitEnd as isize) + 0x0050, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_15_2 = (UnitFields::UnitEnd as isize) + 0x0051, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_15_3 = (UnitFields::UnitEnd as isize) + 0x0052, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_15_5 = (UnitFields::UnitEnd as isize) + 0x0054, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_16_1 = (UnitFields::UnitEnd as isize) + 0x0055, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_16_2 = (UnitFields::UnitEnd as isize) + 0x0056, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_16_3 = (UnitFields::UnitEnd as isize) + 0x0057, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_16_5 = (UnitFields::UnitEnd as isize) + 0x0059, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_17_1 = (UnitFields::UnitEnd as isize) + 0x005A, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_17_2 = (UnitFields::UnitEnd as isize) + 0x005B, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_17_3 = (UnitFields::UnitEnd as isize) + 0x005C, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_17_5 = (UnitFields::UnitEnd as isize) + 0x005E, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_18_1 = (UnitFields::UnitEnd as isize) + 0x005F, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_18_2 = (UnitFields::UnitEnd as isize) + 0x0060, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_18_3 = (UnitFields::UnitEnd as isize) + 0x0061, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_18_5 = (UnitFields::UnitEnd as isize) + 0x0063, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_19_1 = (UnitFields::UnitEnd as isize) + 0x0064, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_19_2 = (UnitFields::UnitEnd as isize) + 0x0065, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_19_3 = (UnitFields::UnitEnd as isize) + 0x0066, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_19_5 = (UnitFields::UnitEnd as isize) + 0x0068, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_20_1 = (UnitFields::UnitEnd as isize) + 0x0069, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_20_2 = (UnitFields::UnitEnd as isize) + 0x006A, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_20_3 = (UnitFields::UnitEnd as isize) + 0x006B, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_20_5 = (UnitFields::UnitEnd as isize) + 0x006D, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_21_1 = (UnitFields::UnitEnd as isize) + 0x006E, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_21_2 = (UnitFields::UnitEnd as isize) + 0x006F, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_21_3 = (UnitFields::UnitEnd as isize) + 0x0070, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_21_5 = (UnitFields::UnitEnd as isize) + 0x0072, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_22_1 = (UnitFields::UnitEnd as isize) + 0x0073, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_22_2 = (UnitFields::UnitEnd as isize) + 0x0074, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_22_3 = (UnitFields::UnitEnd as isize) + 0x0075, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_22_5 = (UnitFields::UnitEnd as isize) + 0x0077, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_23_1 = (UnitFields::UnitEnd as isize) + 0x0078, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_23_2 = (UnitFields::UnitEnd as isize) + 0x0079, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_23_3 = (UnitFields::UnitEnd as isize) + 0x007A, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_23_5 = (UnitFields::UnitEnd as isize) + 0x007C, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_24_1 = (UnitFields::UnitEnd as isize) + 0x007D, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_24_2 = (UnitFields::UnitEnd as isize) + 0x007E, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_24_3 = (UnitFields::UnitEnd as isize) + 0x007F, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_24_5 = (UnitFields::UnitEnd as isize) + 0x0081, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_25_1 = (UnitFields::UnitEnd as isize) + 0x0082, // Size: 1, Type: Int, Flags: PartyMember
    PlayerQuestLog_25_2 = (UnitFields::UnitEnd as isize) + 0x0083, // Size: 1, Type: Int, Flags: Private
    PlayerQuestLog_25_3 = (UnitFields::UnitEnd as isize) + 0x0084, // Size: 2, Type: TwoShort, Flags: Private
    PlayerQuestLog_25_5 = (UnitFields::UnitEnd as isize) + 0x0086, // Size: 1, Type: Int, Flags: Private
    PlayerVisibleItem_1Entryid = (UnitFields::UnitEnd as isize) + 0x0087, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_1Enchantment = (UnitFields::UnitEnd as isize) + 0x0088, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_2Entryid = (UnitFields::UnitEnd as isize) + 0x0089, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_2Enchantment = (UnitFields::UnitEnd as isize) + 0x008A, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_3Entryid = (UnitFields::UnitEnd as isize) + 0x008B, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_3Enchantment = (UnitFields::UnitEnd as isize) + 0x008C, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_4Entryid = (UnitFields::UnitEnd as isize) + 0x008D, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_4Enchantment = (UnitFields::UnitEnd as isize) + 0x008E, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_5Entryid = (UnitFields::UnitEnd as isize) + 0x008F, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_5Enchantment = (UnitFields::UnitEnd as isize) + 0x0090, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_6Entryid = (UnitFields::UnitEnd as isize) + 0x0091, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_6Enchantment = (UnitFields::UnitEnd as isize) + 0x0092, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_7Entryid = (UnitFields::UnitEnd as isize) + 0x0093, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_7Enchantment = (UnitFields::UnitEnd as isize) + 0x0094, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_8Entryid = (UnitFields::UnitEnd as isize) + 0x0095, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_8Enchantment = (UnitFields::UnitEnd as isize) + 0x0096, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_9Entryid = (UnitFields::UnitEnd as isize) + 0x0097, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_9Enchantment = (UnitFields::UnitEnd as isize) + 0x0098, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_10Entryid = (UnitFields::UnitEnd as isize) + 0x0099, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_10Enchantment = (UnitFields::UnitEnd as isize) + 0x009A, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_11Entryid = (UnitFields::UnitEnd as isize) + 0x009B, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_11Enchantment = (UnitFields::UnitEnd as isize) + 0x009C, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_12Entryid = (UnitFields::UnitEnd as isize) + 0x009D, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_12Enchantment = (UnitFields::UnitEnd as isize) + 0x009E, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_13Entryid = (UnitFields::UnitEnd as isize) + 0x009F, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_13Enchantment = (UnitFields::UnitEnd as isize) + 0x00A0, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_14Entryid = (UnitFields::UnitEnd as isize) + 0x00A1, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_14Enchantment = (UnitFields::UnitEnd as isize) + 0x00A2, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_15Entryid = (UnitFields::UnitEnd as isize) + 0x00A3, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_15Enchantment = (UnitFields::UnitEnd as isize) + 0x00A4, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_16Entryid = (UnitFields::UnitEnd as isize) + 0x00A5, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_16Enchantment = (UnitFields::UnitEnd as isize) + 0x00A6, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_17Entryid = (UnitFields::UnitEnd as isize) + 0x00A7, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_17Enchantment = (UnitFields::UnitEnd as isize) + 0x00A8, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_18Entryid = (UnitFields::UnitEnd as isize) + 0x00A9, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_18Enchantment = (UnitFields::UnitEnd as isize) + 0x00Aa, // Size: 1, Type: TwoShort, Flags: Public
    PlayerVisibleItem_19Entryid = (UnitFields::UnitEnd as isize) + 0x00Ab, // Size: 1, Type: Int, Flags: Public
    PlayerVisibleItem_19Enchantment = (UnitFields::UnitEnd as isize) + 0x00Ac, // Size: 1, Type: TwoShort, Flags: Public
    PlayerChosenTitle = (UnitFields::UnitEnd as isize) + 0x00Ad, // Size: 1, Type: Int, Flags: Public
    PlayerFakeInebriation = (UnitFields::UnitEnd as isize) + 0x00Ae, // Size: 1, Type: Int, Flags: Public
    Pad_0 = (UnitFields::UnitEnd as isize) + 0x00Af, // Size: 1, Type: Int, Flags: None
    InvSlotHead = (UnitFields::UnitEnd as isize) + 0x00B0, // Size: 46, Type: Long, Flags: Private
    PackSlot_1 = (UnitFields::UnitEnd as isize) + 0x00De, // Size: 32, Type: Long, Flags: Private
    BankSlot_1 = (UnitFields::UnitEnd as isize) + 0x00Fe, // Size: 56, Type: Long, Flags: Private
    BankbagSlot_1 = (UnitFields::UnitEnd as isize) + 0x0136, // Size: 14, Type: Long, Flags: Private
    VendorbuybackSlot_1 = (UnitFields::UnitEnd as isize) + 0x0144, // Size: 24, Type: Long, Flags: Private
    KeyringSlot_1 = (UnitFields::UnitEnd as isize) + 0x015C, // Size: 64, Type: Long, Flags: Private
    CurrencytokenSlot_1 = (UnitFields::UnitEnd as isize) + 0x019C, // Size: 64, Type: Long, Flags: Private
    PlayerFarsight = (UnitFields::UnitEnd as isize) + 0x01Dc, // Size: 2, Type: Long, Flags: Private
    PlayerFieldKnownTitles = (UnitFields::UnitEnd as isize) + 0x01De, // Size: 2, Type: Long, Flags: Private
    PlayerFieldKnownTitles1 = (UnitFields::UnitEnd as isize) + 0x01E0, // Size: 2, Type: Long, Flags: Private
    PlayerFieldKnownTitles2 = (UnitFields::UnitEnd as isize) + 0x01E2, // Size: 2, Type: Long, Flags: Private
    KnownCurrencies = (UnitFields::UnitEnd as isize) + 0x01E4, // Size: 2, Type: Long, Flags: Private
    PlayerXp = (UnitFields::UnitEnd as isize) + 0x01E6,        // Size: 1, Type: Int, Flags: Private
    PlayerNextLevelXp = (UnitFields::UnitEnd as isize) + 0x01E7, // Size: 1, Type: Int, Flags: Private
    PlayerSkillInfo_1_1 = (UnitFields::UnitEnd as isize) + 0x01E8, // Size: 384, Type: TwoShort, Flags: Private
    PlayerCharacterPoints1 = (UnitFields::UnitEnd as isize) + 0x0368, // Size: 1, Type: Int, Flags: Private
    PlayerCharacterPoints2 = (UnitFields::UnitEnd as isize) + 0x0369, // Size: 1, Type: Int, Flags: Private
    PlayerTrackCreatures = (UnitFields::UnitEnd as isize) + 0x036A, // Size: 1, Type: Int, Flags: Private
    PlayerTrackResources = (UnitFields::UnitEnd as isize) + 0x036B, // Size: 1, Type: Int, Flags: Private
    PlayerBlockPercentage = (UnitFields::UnitEnd as isize) + 0x036C, // Size: 1, Type: Float, Flags: Private
    PlayerDodgePercentage = (UnitFields::UnitEnd as isize) + 0x036D, // Size: 1, Type: Float, Flags: Private
    PlayerParryPercentage = (UnitFields::UnitEnd as isize) + 0x036E, // Size: 1, Type: Float, Flags: Private
    PlayerExpertise = (UnitFields::UnitEnd as isize) + 0x036F, // Size: 1, Type: Int, Flags: Private
    PlayerOffhandExpertise = (UnitFields::UnitEnd as isize) + 0x0370, // Size: 1, Type: Int, Flags: Private
    PlayerCritPercentage = (UnitFields::UnitEnd as isize) + 0x0371, // Size: 1, Type: Float, Flags: Private
    PlayerRangedCritPercentage = (UnitFields::UnitEnd as isize) + 0x0372, // Size: 1, Type: Float, Flags: Private
    PlayerOffhandCritPercentage = (UnitFields::UnitEnd as isize) + 0x0373, // Size: 1, Type: Float, Flags: Private
    PlayerSpellCritPercentage1 = (UnitFields::UnitEnd as isize) + 0x0374, // Size: 7, Type: Float, Flags: Private
    PlayerShieldBlock = (UnitFields::UnitEnd as isize) + 0x037B, // Size: 1, Type: Int, Flags: Private
    PlayerShieldBlockCritPercentage = (UnitFields::UnitEnd as isize) + 0x037C, // Size: 1, Type: Float, Flags: Private
    PlayerExploredZones_1 = (UnitFields::UnitEnd as isize) + 0x037D, // Size: 128, Type: Bytes, Flags: Private
    PlayerRestStateExperience = (UnitFields::UnitEnd as isize) + 0x03Fd, // Size: 1, Type: Int, Flags: Private
    Coinage = (UnitFields::UnitEnd as isize) + 0x03Fe, // Size: 1, Type: Int, Flags: Private
    ModDamageDonePos = (UnitFields::UnitEnd as isize) + 0x03Ff, // Size: 7, Type: Int, Flags: Private
    ModDamageDoneNeg = (UnitFields::UnitEnd as isize) + 0x0406, // Size: 7, Type: Int, Flags: Private
    ModDamageDonePct = (UnitFields::UnitEnd as isize) + 0x040D, // Size: 7, Type: Int, Flags: Private
    ModHealingDonePos = (UnitFields::UnitEnd as isize) + 0x0414, // Size: 1, Type: Int, Flags: Private
    ModHealingPct = (UnitFields::UnitEnd as isize) + 0x0415, // Size: 1, Type: Float, Flags: Private
    ModHealingDonePct = (UnitFields::UnitEnd as isize) + 0x0416, // Size: 1, Type: Float, Flags: Private
    ModTargetResistance = (UnitFields::UnitEnd as isize) + 0x0417, // Size: 1, Type: Int, Flags: Private
    ModTargetPhysicalResistance = (UnitFields::UnitEnd as isize) + 0x0418, // Size: 1, Type: Int, Flags: Private
    Bytes = (UnitFields::UnitEnd as isize) + 0x0419, // Size: 1, Type: Bytes, Flags: Private
    PlayerAmmoId = (UnitFields::UnitEnd as isize) + 0x041A, // Size: 1, Type: Int, Flags: Private
    PlayerSelfResSpell = (UnitFields::UnitEnd as isize) + 0x041B, // Size: 1, Type: Int, Flags: Private
    PvpMedals = (UnitFields::UnitEnd as isize) + 0x041C, // Size: 1, Type: Int, Flags: Private
    BuybackPrice_1 = (UnitFields::UnitEnd as isize) + 0x041D, // Size: 12, Type: Int, Flags: Private
    BuybackTimestamp_1 = (UnitFields::UnitEnd as isize) + 0x0429, // Size: 12, Type: Int, Flags: Private
    Kills = (UnitFields::UnitEnd as isize) + 0x0435, // Size: 1, Type: TwoShort, Flags: Private
    TodayContribution = (UnitFields::UnitEnd as isize) + 0x0436, // Size: 1, Type: Int, Flags: Private
    YesterdayContribution = (UnitFields::UnitEnd as isize) + 0x0437, // Size: 1, Type: Int, Flags: Private
    LifetimeHonorbaleKills = (UnitFields::UnitEnd as isize) + 0x0438, // Size: 1, Type: Int, Flags: Private
    Bytes2 = (UnitFields::UnitEnd as isize) + 0x0439, // Size: 1, Type: 6, Flags: Private
    WatchedFactionIndex = (UnitFields::UnitEnd as isize) + 0x043A, // Size: 1, Type: Int, Flags: Private
    CombatRating_1 = (UnitFields::UnitEnd as isize) + 0x043B, // Size: 25, Type: Int, Flags: Private
    ArenaTeamInfo_1_1 = (UnitFields::UnitEnd as isize) + 0x0454, // Size: 21, Type: Int, Flags: Private
    HonorCurrency = (UnitFields::UnitEnd as isize) + 0x0469, // Size: 1, Type: Int, Flags: Private
    ArenaCurrency = (UnitFields::UnitEnd as isize) + 0x046A, // Size: 1, Type: Int, Flags: Private
    MaxLevel = (UnitFields::UnitEnd as isize) + 0x046B,      // Size: 1, Type: Int, Flags: Private
    DailyQuests_1 = (UnitFields::UnitEnd as isize) + 0x046C, // Size: 25, Type: Int, Flags: Private
    PlayerRuneRegen_1 = (UnitFields::UnitEnd as isize) + 0x0485, // Size: 4, Type: Float, Flags: Private
    PlayerNoReagentCost_1 = (UnitFields::UnitEnd as isize) + 0x0489, // Size: 3, Type: Int, Flags: Private
    GlyphSlots_1 = (UnitFields::UnitEnd as isize) + 0x048C, // Size: 6, Type: Int, Flags: Private
    Glyphs_1 = (UnitFields::UnitEnd as isize) + 0x0492,     // Size: 6, Type: Int, Flags: Private
    PlayerGlyphsEnabled = (UnitFields::UnitEnd as isize) + 0x0498, // Size: 1, Type: Int, Flags: Private
    PlayerPetSpellPower = (UnitFields::UnitEnd as isize) + 0x0499, // Size: 1, Type: Int, Flags: Private
    PlayerEnd = (UnitFields::UnitEnd as isize) + 0x049A,
}

#[allow(dead_code)]
pub enum GameObjectFields {
    ObjectFieldCreatedBy = (ObjectFields::End as isize) + 0x0000, // Size: 2, Type: Long, Flags: Public
    Displayid = (ObjectFields::End as isize) + 0x0002, // Size: 1, Type: Int, Flags: Public
    Flags = (ObjectFields::End as isize) + 0x0003,     // Size: 1, Type: Int, Flags: Public
    Parentrotation = (ObjectFields::End as isize) + 0x0004, // Size: 4, Type: Float, Flags: Public
    Dynamic = (ObjectFields::End as isize) + 0x0008,   // Size: 1, Type: TwoShort, Flags: Dynamic
    Faction = (ObjectFields::End as isize) + 0x0009,   // Size: 1, Type: Int, Flags: Public
    Level = (ObjectFields::End as isize) + 0x000A,     // Size: 1, Type: Int, Flags: Public
    Bytes1 = (ObjectFields::End as isize) + 0x000B,    // Size: 1, Type: Bytes, Flags: Public
    End = (ObjectFields::End as isize) + 0x000C,
}

#[allow(dead_code)]
pub enum DynamicObjectFields {
    Caster = (ObjectFields::End as isize) + 0x0000, // Size: 2, Type: Long, Flags: Public
    Bytes = (ObjectFields::End as isize) + 0x0002,  // Size: 1, Type: Bytes, Flags: Public
    Spellid = (ObjectFields::End as isize) + 0x0003, // Size: 1, Type: Int, Flags: Public
    Radius = (ObjectFields::End as isize) + 0x0004, // Size: 1, Type: Float, Flags: Public
    Casttime = (ObjectFields::End as isize) + 0x0005, // Size: 1, Type: Int, Flags: Public
    End = (ObjectFields::End as isize) + 0x0006,
}

#[allow(dead_code)]
pub enum CorpseFields {
    Owner = (ObjectFields::End as isize) + 0x0000, // Size: 2, Type: Long, Flags: Public
    Party = (ObjectFields::End as isize) + 0x0002, // Size: 2, Type: Long, Flags: Public
    DisplayId = (ObjectFields::End as isize) + 0x0004, // Size: 1, Type: Int, Flags: Public
    Item = (ObjectFields::End as isize) + 0x0005,  // Size: 19, Type: Int, Flags: Public
    Bytes1 = (ObjectFields::End as isize) + 0x0018, // Size: 1, Type: Bytes, Flags: Public
    Bytes2 = (ObjectFields::End as isize) + 0x0019, // Size: 1, Type: Bytes, Flags: Public
    Guild = (ObjectFields::End as isize) + 0x001A, // Size: 1, Type: Int, Flags: Public
    Flags = (ObjectFields::End as isize) + 0x001B, // Size: 1, Type: Int, Flags: Public
    DynamicFlags = (ObjectFields::End as isize) + 0x001C, // Size: 1, Type: Int, Flags: Dynamic
    Pad = (ObjectFields::End as isize) + 0x001D,   // Size: 1, Type: Int, Flags: None
    End = (ObjectFields::End as isize) + 0x001E,
}