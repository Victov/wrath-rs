use crate::data::{DataStorage, TutorialFlags, WorldZoneLocation};
use crate::prelude::*;
use crate::world::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use wow_dbc::Indexable;
use wow_world_messages::wrath::{Area, Class, Gender, Map, MovementInfo, Power, Race, Vector3d};

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
            self.gameplay_data.set_unit_DISPLAYID(display_id);
            self.gameplay_data.set_unit_NATIVEDISPLAYID(display_id);
        }

        let class_info = data_storage
            .get_dbc_chr_classes()?
            .get(class.as_int())
            .ok_or_else(|| anyhow!("No classinfo for this class"))?;

        let power = Power::try_from(class_info.display_power as u8)?;
        self.gameplay_data.set_unit_BYTES_0(race, class, gender, power);
        self.gameplay_data.set_unit_HEALTH(100);
        self.gameplay_data.set_unit_MAXHEALTH(100);
        self.gameplay_data.set_unit_LEVEL(1);
        self.gameplay_data.set_unit_FACTIONTEMPLATE(1);
        self.gameplay_data.set_object_SCALE_X(1.0f32);

        //No playtime means it's our very first login
        self.needs_first_login = self.seconds_played_total == 0;

        Ok(())
    }
}
