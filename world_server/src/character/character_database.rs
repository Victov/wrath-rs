use crate::data::{DataStorage, TutorialFlags, WorldZoneLocation};
use crate::packet::ServerMessageExt;
use crate::prelude::*;
use crate::world::prelude::*;
use crate::item::Item;
use std::time::{SystemTime, UNIX_EPOCH};
use wow_dbc::Indexable;
use wow_world_messages::wrath::{Area, Class, Gender, Map, MovementInfo, Power, Race, Vector3d, SkillInfoIndex,SkillInfo, SMSG_INITIAL_SPELLS, InitialSpell, SMSG_UPDATE_OBJECT, Object, Object_UpdateType, UpdateItemBuilder, UpdateMask, MovementBlock, MovementBlock_UpdateFlag, UpdatePlayerBuilder, SMSG_ITEM_PUSH_RESULT};
use wow_world_base::wrath::{RaceClass, ObjectType, ItemSlot, NewItemChatAlert, NewItemCreationType, NewItemSource};
impl super::Character {
    pub(super) async fn load_from_database_internal(&mut self, world: &World, data_storage: &DataStorage) -> Result<()> {
        let character_id = self.get_guid().guid() as u32;
        let realm_database = world.get_realm_database();

        let db_entry = realm_database.get_character(character_id).await?;

        //We don't properly store this in the DB, so try_from will fail because it's always 0
        let bind_area = Area::try_from(db_entry.bind_zone as u32).unwrap_or(Area::NorthshireAbbey);

        self.bind_location = Some(WorldZoneLocation {
            map: Map::try_from(db_entry.bind_map as u32)?,
            area: bind_area,
            position: Vector3d {
                x: db_entry.bind_x,
                y: db_entry.bind_y,
                z: db_entry.bind_z,
            },
            orientation: 0.0, //store in DB?
        });

        self.map = Map::try_from(db_entry.map as u32)?;

        //We don't set this field properly in character creation so consequently its wrong here
        self.area = Area::try_from(db_entry.zone as u32).unwrap_or(Area::NorthshireAbbey);

        self.movement_info = MovementInfo {
            position: Vector3d {
                x: db_entry.x,
                y: db_entry.y,
                z: db_entry.z,
            },
            ..Default::default()
        };

        self.name = db_entry.name.clone();

        self.tutorial_flags = TutorialFlags::from_database_entry(&db_entry)?;
        let character_account_data = realm_database.get_character_account_data(character_id).await?;

        if character_account_data.is_empty() {
            handlers::create_empty_character_account_data_rows(&realm_database, character_id).await?;
        }

        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
        self.last_playtime_calculation_timestamp = unix_time;
        self.seconds_played_total = db_entry.playtime_total;
        self.seconds_played_at_level = db_entry.playtime_level;

        let gender = Gender::try_from(db_entry.gender)?;
        let race = Race::try_from(db_entry.race)?;
        let class = Class::try_from(db_entry.class)?;

        if let Some(race_info) = data_storage.get_dbc_chr_races()?.get(race.as_int()) {
            let display_id = match gender {
                Gender::Male => race_info.male_display_id,
                _ => race_info.female_display_id,
            }
            .id;
            self.gameplay_data.set_unit_displayid(display_id);
            self.gameplay_data.set_unit_nativedisplayid(display_id);
        }

        let class_info = data_storage
            .get_dbc_chr_classes()?
            .get(class.as_int())
            .ok_or_else(|| anyhow!("No classinfo for this class"))?;

        let power = Power::try_from(class_info.display_power as u8)?;
        self.gameplay_data.set_unit_bytes_0(race, class, gender, power);
        self.gameplay_data.set_unit_health(100);
        self.gameplay_data.set_unit_maxhealth(100);
        self.gameplay_data.set_unit_level(1);
        self.gameplay_data.set_unit_factiontemplate(1);
        self.gameplay_data.set_object_scale_x(1.0f32);

        //No playtime means it's our very first login
        self.needs_first_login = self.seconds_played_total == 0;

        //TODO: this should be loaded from the DB, it's a placeholder
        let race_class = RaceClass::try_from((race,class)).unwrap();
        for (i, skill) in race_class.starter_skills().iter().enumerate(){
            self.gameplay_data.set_player_skill_info(
                SkillInfo::new(*skill, 0,1, 300, 0, 0),
                SkillInfoIndex::try_from(i as u32).unwrap(),
            );
        }
        //TODO: learning some skills might learn spells, those need to be checked too?


        //TODO: this should be loaded from the DB, it's a placeholder
        SMSG_INITIAL_SPELLS{
             unknown1: 0,
             initial_spells: race_class.starter_spells().iter().map(|x| InitialSpell{spell_id: *x, unknown1: 0,}).collect(),
             cooldowns: vec![] }.astd_send_to_character(&mut *self).await?;

        //TODO: load invetory here
        realm_database.get_all_character_equipment(character_id).await?.iter().for_each(|x|{
            self.items.try_insert_item(Item::from(x)).unwrap();
        });

        let char_equipment = self.items.get_all_equipment();
        let equiped_items = char_equipment
                .iter()
                .filter_map(|x| *x)
                .map(|x| Object {
                update_type: Object_UpdateType::CreateObject {
                    guid3: x.update_state.object_guid().unwrap(),
                    mask2: UpdateMask::Item(x.update_state.clone()),
                    movement2: MovementBlock {
                        update_flag: MovementBlock_UpdateFlag::empty()
                    },
                    object_type: ObjectType::Item,
                },
                }).collect::<Vec<Object>>();

        char_equipment.iter().enumerate().for_each(|(i,x)|{
            if let Some(item) = x{
                self.gameplay_data.set_player_field_inv(ItemSlot::try_from(i as u8).unwrap(),item.update_state.object_guid().unwrap());
            }
        });
        SMSG_UPDATE_OBJECT {
              objects: equiped_items,
            }.astd_send_to_character(&mut *self).await?;

        Ok(())
    }
}
