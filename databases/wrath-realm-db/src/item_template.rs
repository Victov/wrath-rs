use std::convert::TryInto;

use anyhow::Result;

#[derive(Default)]
pub struct DBItemSkillRequirement {
    pub skill_id: u16,
    pub required_rank: u16,
}

#[derive(Default)]
pub struct DBItemFactionRequirement {
    pub faction_id: u16,
    pub required_rank: u16,
}

pub struct DBItemStat {
    pub stat_type: u8,
    pub stat_value: u16,
}

#[derive(Default, Clone)]
pub struct DBItemDamage {
    pub min: f32,
    pub max: f32,
    pub damage_type: u8,
}

#[derive(Default, Clone)]
pub struct DBItemResistances {
    pub holy: u8,
    pub fire: u8,
    pub nature: u8,
    pub frost: u8,
    pub shadow: u8,
    pub arcane: u8,
}

//Don't derive Default, it's special, down below
#[derive(Clone)]
pub struct DBItemSpellProc {
    pub spell_id: u32,
    pub trigger_type: u8,
    pub charges: u16,
    pub procs_per_minute: f32,
    pub cooldown: u32, //in ms
    pub category: u16,
    pub category_cooldown: u32, //in ms
}

impl Default for DBItemSpellProc {
    fn default() -> Self {
        Self {
            spell_id: 0,
            trigger_type: 0,
            charges: 0,
            cooldown: u32::max_value(),
            procs_per_minute: 0.0f32,
            category: 0,
            category_cooldown: u32::max_value(),
        }
    }
}

#[derive(Default)]
pub struct DBItemReadableInfo {
    pub text_id: u32,
    pub language_id: u8,
    pub page_material: u8,
}

#[derive(Default, Clone)]
pub struct DBItemSocketInfo {
    pub color: u8,
    pub content: u32,
}

pub struct DBItemContainerLootInfo {
    pub money_loot_min: u32,
    pub money_loot_max: u32,
}

pub struct DBItemTemplate {
    pub id: u32,
    pub class: u8,
    pub subclass: u8,
    pub name: String,
    pub displayid: u32,
    pub quality: u8,
    pub flags: u32,
    pub flags2: u32,
    pub buy_count: u8,
    pub buy_price: u32,
    pub sell_price: u32,
    pub inventory_type: u8,
    pub allowed_classes_mask: Option<u32>, //None means allowed by anything
    pub allowed_races_mask: Option<u32>,   //None means allowed by anything
    pub item_level: u16,
    pub required_level: Option<u8>, //None means no requirement
    pub required_skill: Option<DBItemSkillRequirement>,
    pub required_spell_id: Option<u32>,
    pub required_honor_rank: Option<u32>,
    pub required_faction: Option<DBItemFactionRequirement>,
    pub max_count: i16,
    pub stackable: i16,
    pub container_slots: u8,
    pub granted_stats: Vec<DBItemStat>,
    pub scaling_stat_distribution: i16,
    pub scaling_stat_value: u32,
    pub damage: Vec<DBItemDamage>,
    pub granted_armor: Option<u16>,
    pub granted_resistances: Option<DBItemResistances>,
    pub delay: Option<u16>, //Time between attacks in ms
    pub ammo_type: Option<u8>,
    pub ranged_mod_range: f32,
    pub spell_procs: Vec<DBItemSpellProc>,
    pub bonding: u8,
    pub description: String,
    pub readable_info: Option<DBItemReadableInfo>,
    pub start_quest_id: Option<u32>,
    pub lock_id: Option<u32>,
    pub material: i8,
    pub sheath_style: u8,
    pub random_property: u32,
    pub random_suffix: u32,
    pub block_value: Option<u32>,
    pub item_set_id: Option<u32>,
    pub max_durability: u16,
    pub usable_area: Option<u32>,
    pub usable_map: Option<u16>,
    pub bag_family_mask: Option<i32>,
    pub totem_category: Option<i32>,
    pub sockets: Vec<DBItemSocketInfo>,
    pub socket_bonus: Option<u32>,
    pub gem_properties: u32,
    pub required_disenchant_skill: Option<i16>,
    pub armor_damage_modifier: f32,
    pub duration: u32,
    pub item_limit_category: u16,
    pub holiday_id: u32,
    pub disenchant_id: u32,
    pub food_type: u8,
    pub container_loot_info: Option<DBItemContainerLootInfo>,
    pub extra_flags: u8,
}

impl super::RealmDatabase {
    pub async fn get_item_template(&self, item_id: u32) -> Result<DBItemTemplate> {
        let res = sqlx::query!("SELECT * FROM item_template WHERE id = ?", item_id,)
            .fetch_one(&self.connection_pool)
            .await?;

        let item = DBItemTemplate {
            id: res.id,
            class: res.class,
            subclass: res.class,
            name: res.name,
            displayid: res.displayid,
            quality: res.Quality,
            flags: res.Flags,
            flags2: res.Flags2,
            buy_count: res.BuyCount,
            buy_price: res.BuyPrice,
            sell_price: res.SellPrice,
            inventory_type: res.inventory_type,
            allowed_races_mask: match res.AllowableRace {
                -1 => None,
                val => Some(val.try_into().unwrap_or(u32::max_value())),
            },
            allowed_classes_mask: match res.AllowableClass {
                -1 => None,
                val => Some(val.try_into().unwrap_or(u32::max_value())),
            },
            item_level: res.ItemLevel,
            required_level: match res.RequiredLevel {
                0 => None,
                val => Some(val),
            },
            required_skill: match res.RequiredSkill {
                0 => None,
                v => Some(DBItemSkillRequirement {
                    skill_id: v,
                    required_rank: res.RequiredSkillRank,
                }),
            },
            required_spell_id: match res.requiredspell {
                0 => None,
                v => Some(v),
            },
            required_honor_rank: match res.requiredhonorrank {
                0 => None,
                v => Some(v),
            },
            required_faction: match res.RequiredReputationFaction {
                0 => None,
                v => Some(DBItemFactionRequirement {
                    faction_id: v,
                    required_rank: res.RequiredReputationRank,
                }),
            },
            max_count: res.maxcount,
            stackable: res.stackable,
            container_slots: res.ContainerSlots,
            granted_stats: vec![],
            scaling_stat_distribution: res.ScalingStatDistribution,
            scaling_stat_value: res.ScalingStatValue,
            damage: vec![],
            granted_armor: match res.armor {
                0 => None,
                v => Some(v),
            },
            granted_resistances: if res.holy_res != 0
                || res.fire_res != 0
                || res.nature_res != 0
                || res.frost_res != 0
                || res.shadow_res != 0
                || res.arcane_res != 0
            {
                Some(DBItemResistances {
                    holy: res.holy_res,
                    fire: res.fire_res,
                    nature: res.nature_res,
                    frost: res.frost_res,
                    shadow: res.shadow_res,
                    arcane: res.arcane_res,
                })
            } else {
                None
            },
            delay: match res.delay {
                0 => None,
                v => Some(v),
            },
            ammo_type: match res.ammo_type {
                0 => None,
                v => Some(v),
            },
            ranged_mod_range: res.RangedModRange,
            spell_procs: vec![],
            bonding: res.bonding,
            description: res.description,
            readable_info: match res.PageText {
                0 => None,
                v => Some(DBItemReadableInfo {
                    text_id: v,
                    language_id: res.LanguageID,
                    page_material: res.PageMaterial,
                }),
            },
            start_quest_id: match res.startquest {
                0 => None,
                v => Some(v),
            },
            lock_id: match res.lockid {
                0 => None,
                v => Some(v),
            },
            material: res.Material,
            sheath_style: res.sheath,
            random_property: res.RandomProperty,
            random_suffix: res.RandomSuffix,
            block_value: match res.block {
                0 => None,
                v => Some(v),
            },
            item_set_id: match res.itemset {
                0 => None,
                v => Some(v),
            },
            max_durability: res.MaxDurability,
            usable_area: match res.area {
                0 => None,
                v => Some(v),
            },
            usable_map: match res.Map {
                0 => None,
                v => Some(v),
            },
            bag_family_mask: match res.BagFamily {
                0 => None,
                v => Some(v),
            },
            totem_category: match res.TotemCategory {
                0 => None,
                v => Some(v),
            },
            sockets: vec![],
            socket_bonus: match res.socketBonus {
                0 => None,
                v => Some(v),
            },
            gem_properties: res.GemProperties,
            required_disenchant_skill: match res.RequiredDisenchantSkill {
                -1 => None,
                v => Some(v),
            },
            armor_damage_modifier: res.ArmorDamageModifier,
            duration: res.Duration,
            item_limit_category: res.ItemLimitCategory,
            holiday_id: res.HolidayId,
            disenchant_id: res.DisenchantID,
            food_type: res.FoodType,
            container_loot_info: match res.minMoneyLoot {
                0 => None,
                v => Some(DBItemContainerLootInfo {
                    money_loot_min: v,
                    money_loot_max: res.maxMoneyLoot,
                }),
            },
            extra_flags: res.ExtraFlags,
        };

        //TODO: fill vecs granted_stats, damage, spell_procs and sockets

        Ok(item)
    }
}
