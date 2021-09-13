use super::world::prelude::*;
use crate::client::Client;
use crate::constants::social::RelationType;
use crate::data_types::{ActionBar, PositionAndOrientation, TutorialFlags, WorldZoneLocation};
use crate::guid::*;
use crate::world::prelude::updates::ObjectType;
use crate::ClientManager;
use anyhow::{bail, Result};
use async_std::sync::RwLock;
use std::sync::Weak;
use wrath_realm_db::RealmDatabase;

const NUM_UNIT_FIELDS: usize = PlayerFields::PlayerEnd as usize;

pub struct Character {
    pub guid: Guid,
    pub client: Weak<RwLock<Client>>,

    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation: f32,
    pub map: u32,
    pub instance_id: u32,
    pub bind_location: WorldZoneLocation,
    pub tutorial_flags: TutorialFlags,
    pub action_bar: ActionBar,

    //required for world updates and implenting ReceiveUpdates trait
    creation_buffer: Vec<u8>,
    creation_block_count: u32,

    //required for unit values and implementing ValueFieldsRaw trait, which in turn will grant us
    //HasvalueFields trait, with all sorts of goodies
    unit_value_fields: [u32; NUM_UNIT_FIELDS],
}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.guid == other.guid
    }
}

impl Character {
    pub async fn load_from_database(client: Weak<RwLock<Client>>, realm_database: &RealmDatabase, guid: Guid) -> Result<Self> {
        let db_entry = realm_database.get_character(guid.get_low_part()).await?;
        let bind_location = WorldZoneLocation {
            zone: db_entry.bind_zone as u32,
            map: db_entry.bind_map as u32,
            x: db_entry.bind_x,
            y: db_entry.bind_y,
            z: db_entry.bind_z,
        };

        let tutorial_flags = TutorialFlags::from_database_entry(&db_entry)?;

        let character_id = guid.get_low_part();
        let character_account_data = realm_database.get_character_account_data(character_id).await?;

        if character_account_data.len() == 0 {
            crate::handlers::create_empty_character_account_data_rows(realm_database, character_id).await?;
        }

        Ok(Self {
            guid,
            client,
            x: db_entry.x,
            y: db_entry.y,
            z: db_entry.z,
            orientation: 0f32,
            map: db_entry.map as u32,
            instance_id: db_entry.instance_id,
            bind_location,
            tutorial_flags,
            action_bar: ActionBar::new(), //TODO: store/read from database
            creation_block_count: 0,
            creation_buffer: vec![],
            unit_value_fields: [0; NUM_UNIT_FIELDS],
        })
    }

    pub async fn perform_login(&mut self, client_manager: &ClientManager) -> Result<()> {
        let world = client_manager.world.clone();

        crate::handlers::send_verify_world(&self).await?;
        crate::handlers::send_dungeon_difficulty(&self).await?;
        crate::handlers::send_character_account_data_times(client_manager, &self).await?;
        crate::handlers::send_voice_chat_status(&self, false).await?;
        crate::handlers::send_bind_update(&self).await?;
        crate::handlers::send_tutorial_flags(&self).await?;
        crate::handlers::send_login_set_time_speed(&self).await?;
        crate::handlers::send_action_buttons(&self).await?;
        crate::handlers::send_faction_list(&self).await?;
        crate::handlers::send_initial_spells(&self).await?;
        crate::handlers::send_talents_info(&self).await?;
        crate::handlers::send_aura_update_all(&self).await?;
        crate::handlers::send_contact_list(&self, &[RelationType::Friend, RelationType::Muted, RelationType::Ignore]).await?;
        crate::handlers::send_initial_world_states(&self).await?;
        //crate::handlers::send_world_state_update(&self, 0xF3D, 0).await?;
        //crate::handlers::send_world_state_update(&self, 0xC77, 0).await?;
        //crate::handlers::send_temp_dummy_hardcoded_update(&self).await?;

        let race = 1u32; //human
        let class = 1u32; //warrior
        let gender = 1u32; //female
        let power_type = 1u32; //rage

        self.set_object_field_u32(ObjectFields::LowGuid, self.get_guid().get_low_part())?;
        self.set_object_field_u32(ObjectFields::HighGuid, self.get_guid().get_high_part())?;
        self.set_object_field_u32(
            ObjectFields::Type,
            1 << ObjectType::Unit as u32 | 1 << ObjectType::Player as u32 | 1 << ObjectType::Object as u32,
        )?;
        self.set_object_field_f32(ObjectFields::Scale, 1.0f32)?;
        self.set_unit_field_u32(UnitFields::UnitBytes0, (race << 24) | (class << 16) | (gender << 8) | power_type)?;
        self.set_unit_field_u32(UnitFields::Health, 100)?;
        self.set_unit_field_u32(UnitFields::Maxhealth, 100)?;
        self.set_unit_field_u32(UnitFields::Level, 1)?;
        self.set_unit_field_u32(UnitFields::Factiontemplate, 1)?;
        self.set_unit_field_u32(UnitFields::Displayid, 19724)?; //human female
        self.set_unit_field_u32(UnitFields::Nativedisplayid, 19724)?;

        world
            .get_instance_manager()
            .get_map_for_instance(self.instance_id)
            .await
            .read()
            .await
            .push_object(self)
            .await?;

        let (num, buf) = self.get_creation_data();
        super::handlers::send_update_packet(&self, num, &buf).await?;
        self.clear_creation_data();

        Ok(())
    }
}

impl MapObject for Character {
    fn get_guid(&self) -> &Guid {
        &self.guid
    }
    fn set_in_cell(&mut self, _cell: &MapCell) {}
    fn get_type(&self) -> updates::ObjectType {
        updates::ObjectType::Player
    }
    fn get_position(&self) -> PositionAndOrientation {
        PositionAndOrientation {
            x: self.x,
            y: self.y,
            z: self.z,
            o: self.orientation,
        }
    }
}

impl ReceiveUpdates for Character {
    fn push_creation_data(&mut self, data: &mut Vec<u8>, block_count: u32) {
        self.creation_buffer.append(data);
        self.creation_block_count += block_count;
    }
    fn get_creation_data(&self) -> (u32, &Vec<u8>) {
        (self.creation_block_count, &self.creation_buffer)
    }
    fn clear_creation_data(&mut self) {
        self.creation_block_count = 0;
        self.creation_buffer.clear();
    }
}

impl ValueFieldsRaw for Character {
    fn set_field_u32(&mut self, field: usize, value: u32) -> Result<()> {
        if field > self.unit_value_fields.len() {
            bail!("Out-of-range unit field being set")
        }
        self.unit_value_fields[field] = value;
        Ok(())
    }

    fn get_field_u32(&self, field: usize) -> Result<u32> {
        if field > self.unit_value_fields.len() {
            bail!("Out-of-range unit field being accessed");
        }
        Ok(self.unit_value_fields[field])
    }

    fn get_num_value_fields(&self) -> usize {
        NUM_UNIT_FIELDS
    }
}
