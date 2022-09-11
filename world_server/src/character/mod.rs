use super::world::prelude::*;
use crate::client::Client;
use crate::constants::social::RelationType;
use crate::data::{ActionBar, DataStorage, MovementInfo, PositionAndOrientation, TutorialFlags, WorldZoneLocation};
//use crate::handlers::{login_handler::LogoutState, movement_handler::TeleportationState};
use crate::item::Item;
use crate::prelude::*;
use async_std::sync::RwLock;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};
use wrath_realm_db::RealmDatabase;

const NUM_UNIT_FIELDS: usize = PlayerFields::PlayerEnd as usize;

//mod character_equipment;
//mod character_logout;
//mod character_movement;
//mod character_rested;

pub struct Character {
    pub guid: Guid,
    pub client: Weak<Client>,
    pub name: String,
    pub race: u8,
    pub class: u8,
    pub gender: u8,
    pub movement_info: MovementInfo,

    pub map: u32,
    pub zone: u32,
    pub instance_id: u32,
    pub bind_location: Option<WorldZoneLocation>,
    pub tutorial_flags: TutorialFlags,
    pub action_bar: ActionBar,

    //Stuff to keep track of playtime
    pub seconds_played_total: u32,
    pub seconds_played_at_level: u32,
    pub last_playtime_calculation_timestamp: u32,

    //required for world updates and implenting ReceiveUpdates trait
    creation_buffer: Vec<u8>,
    creation_block_count: u32,

    //required for unit values and implementing ValueFieldsRaw trait, which in turn will grant us
    //HasvalueFields trait, with all sorts of goodies
    unit_value_fields: [u32; NUM_UNIT_FIELDS],
    changed_update_mask: UpdateMask,

    //things required to keep MapObject working
    in_range_objects: HashMap<Guid, Weak<RwLock<dyn GameObject>>>,
    recently_removed_guids: Vec<Guid>,

    //time sync
    pub time_sync_counter: u32,
    time_sync_cooldown: f32,

    //Teleporting
    //pub teleportation_state: TeleportationState,

    //pub logout_state: LogoutState,
    //rested_state: character_rested::RestedState,
    equipped_items: Vec<Arc<RwLock<Item>>>,
}

impl Character {
    pub fn new(client: Weak<Client>, guid: Guid) -> Self {
        Self {
            guid,
            client,
            name: String::new(),
            race: 0,
            class: 0,
            gender: 0,
            movement_info: MovementInfo::default(),
            map: 0,
            zone: 0,
            instance_id: 0,
            bind_location: None,
            tutorial_flags: [0; 32].into(),
            action_bar: ActionBar::new(),
            seconds_played_total: 0,
            seconds_played_at_level: 0,
            last_playtime_calculation_timestamp: 0,
            creation_block_count: 0,
            creation_buffer: vec![],
            unit_value_fields: [0; NUM_UNIT_FIELDS],
            changed_update_mask: UpdateMask::new(NUM_UNIT_FIELDS),
            in_range_objects: HashMap::new(),
            recently_removed_guids: vec![],
            time_sync_counter: 0,
            time_sync_cooldown: 0f32,
            //teleportation_state: TeleportationState::None,
            //logout_state: LogoutState::None,
            //rested_state: character_rested::RestedState::NotRested,
            equipped_items: vec![],
        }
    }

    pub async fn load_from_database(&mut self, world: &World, data_storage: &DataStorage) -> Result<()> {
        let realm_database = world.get_realm_database();
        let db_entry = realm_database.get_character(self.guid.get_low_part()).await?;
        self.bind_location = Some(WorldZoneLocation {
            zone: db_entry.bind_zone as u32,
            map: db_entry.bind_map as u32,
            x: db_entry.bind_x,
            y: db_entry.bind_y,
            z: db_entry.bind_z,
            o: 0.0, //store in DB?
        });
        self.map = db_entry.map as u32;
        self.movement_info = MovementInfo {
            position: PositionAndOrientation {
                x: db_entry.x,
                y: db_entry.y,
                z: db_entry.z,
                o: db_entry.o,
            },
            ..Default::default()
        };
        self.name = db_entry.name.clone();

        self.tutorial_flags = TutorialFlags::from_database_entry(&db_entry)?;
        let character_id = self.guid.get_low_part();
        let character_account_data = realm_database.get_character_account_data(character_id).await?;

        if character_account_data.is_empty() {
            //handlers::create_empty_character_account_data_rows(&realm_database, character_id).await?;
        }

        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

        self.last_playtime_calculation_timestamp = unix_time;
        self.seconds_played_total = db_entry.playtime_total;
        self.seconds_played_at_level = db_entry.playtime_level;

        self.gender = db_entry.gender;
        self.race = db_entry.race;
        if let Some(race_info) = data_storage.get_char_races().get_entry(self.race as u32) {
            let display_id = match self.gender {
                0 => race_info.male_model_id,
                _ => race_info.female_model_id,
            };
            self.set_unit_field_u32(UnitFields::Displayid, display_id)?;
            self.set_unit_field_u32(UnitFields::Nativedisplayid, display_id)?;
        }

        self.class = db_entry.class;
        if let Some(class_info) = data_storage.get_char_classes().get_entry(self.class as u32) {
            self.set_power_type(class_info.power_type as u8)?;
        }

        self.set_object_field_u32(ObjectFields::LowGuid, self.get_guid().get_low_part())?;
        self.set_object_field_u32(ObjectFields::HighGuid, self.get_guid().get_high_part())?;
        self.set_object_field_u32(
            ObjectFields::Type,
            1 << ObjectType::Unit as u32 | 1 << ObjectType::Player as u32 | 1 << ObjectType::Object as u32,
        )?;
        self.set_object_field_f32(ObjectFields::Scale, 1.0f32)?;
        self.set_class(self.class)?;
        self.set_race(self.race)?;
        self.set_gender(self.gender)?;
        self.set_unit_field_u32(UnitFields::Health, 100)?;
        self.set_unit_field_u32(UnitFields::Maxhealth, 100)?;
        self.set_unit_field_u32(UnitFields::Level, 1)?;
        self.set_unit_field_u32(UnitFields::Factiontemplate, 1)?;

        //self.load_equipment_from_database(world).await?;

        Ok(())
    }

    pub async fn send_packets_before_add_to_map(&self) -> Result<()> {
        /*handlers::send_contact_list(self, &[RelationType::Friend, RelationType::Muted, RelationType::Ignore]).await?;
        handlers::send_bind_update(self).await?;
        handlers::send_talents_info(self).await?;
        handlers::send_dungeon_difficulty(self).await?;
        handlers::send_initial_spells(self).await?;
        handlers::send_action_buttons(self).await?;
        handlers::send_initial_world_states(self).await?;
        */
        handlers::send_login_set_time_speed(self).await
    }

    pub async fn send_packets_after_add_to_map(&self, realm_database: Arc<RealmDatabase>) -> Result<()> {
        /*
        handlers::send_verify_world(self).await?;
        handlers::send_character_account_data_times(&realm_database, self).await?;
        handlers::send_voice_chat_status(self, false).await?;
        handlers::send_tutorial_flags(self).await?;
        handlers::send_faction_list(self).await?;
        handlers::send_aura_update_all(self).await?;
        handlers::send_time_sync(self).await?;
        //handlers::send_world_state_update(&self, 0xF3D, 0).await?;
        //handlers::send_world_state_update(&self, 0xC77, 0).await?;
        */
        Ok(())
    }

    pub async fn zone_update(&mut self, zone: u32) -> Result<()> {
        if self.zone == zone {
            return Ok(());
        }

        trace!("Received zone update for character {} into zone {}", self.name, zone);
        self.zone = zone;
        //handlers::send_initial_world_states(self).await
        Ok(())
    }

    pub fn reset_time_sync(&mut self) {
        self.time_sync_cooldown = 0.0;
        self.time_sync_counter = 0;
    }
    pub async fn tick(&mut self, delta_time: f32, world: Arc<World>) -> Result<()> {
        self.tick_time_sync(delta_time).await?;
        //self.tick_logout_state(delta_time, world.clone()).await?;

        /*self.handle_queued_teleport(world)
            .await
            .unwrap_or_else(|e| warn!("Could not teleport player {}: Error {}", self.name, e));
        */

        Ok(())
    }
    async fn tick_time_sync(&mut self, delta_time: f32) -> Result<()> {
        self.time_sync_cooldown -= delta_time;
        if self.time_sync_cooldown < 0f32 {
            self.time_sync_cooldown += 10f32;
            self.time_sync_counter += 1;
            //handlers::send_time_sync(self).await?;
        }
        Ok(())
    }

    pub async fn try_get_self_arc(&self) -> Result<Arc<RwLock<Self>>> {
        let client = self
            .client
            .upgrade()
            .ok_or_else(|| anyhow!("Could not get an Arc to the character because the owning client does not exist"))?;
        client.get_active_character().await
    }
}

#[async_trait::async_trait]
impl MapObject for Character {
    fn get_guid(&self) -> &Guid {
        &self.guid
    }
    fn get_type(&self) -> updates::ObjectType {
        ObjectType::Player
    }

    async fn on_pushed_to_map(&mut self, map_manager: &MapManager) -> Result<()> {
        /*
        self.push_create_blocks_for_items(map_manager).await?;
        let (block_count, mut update_data) = build_create_update_block_for_player(self, self)?;
        self.push_update_block(&mut update_data, block_count);
        */
        Ok(())
    }
}

//Implement features for gameobject. For character, almost all features (traits) are enabled (Some)
impl GameObject for Character {
    fn as_update_receiver_mut(&mut self) -> Option<&mut dyn ReceiveUpdates> {
        Some(self)
    }

    fn as_update_receiver(&self) -> Option<&dyn ReceiveUpdates> {
        Some(self)
    }

    fn as_world_object_mut(&mut self) -> Option<&mut dyn WorldObject> {
        Some(self)
    }

    fn as_world_object(&self) -> Option<&dyn WorldObject> {
        Some(self)
    }

    fn as_map_object_mut(&mut self) -> &mut dyn MapObject {
        self
    }

    fn as_map_object(&self) -> &dyn MapObject {
        self
    }

    fn as_character(&self) -> Option<&Character> {
        Some(self)
    }

    fn as_has_value_fields(&self) -> Option<&dyn HasValueFields> {
        Some(self)
    }

    fn as_has_value_fields_mut(&mut self) -> Option<&mut dyn HasValueFields> {
        Some(self)
    }
}

impl WorldObject for Character {
    fn get_position(&self) -> &PositionAndOrientation {
        &self.movement_info.position
    }

    fn get_movement_info(&self) -> &MovementInfo {
        &self.movement_info
    }

    fn is_in_range(&self, guid: &Guid) -> bool {
        self.in_range_objects.contains_key(guid)
    }

    fn add_in_range_object(&mut self, guid: &Guid, object: Weak<RwLock<dyn GameObject>>) -> Result<()> {
        assert!(!self.is_in_range(guid));
        self.in_range_objects.insert(*guid, object);
        Ok(())
    }

    fn get_in_range_guids(&self) -> Vec<&Guid> {
        self.in_range_objects.keys().collect()
    }

    fn remove_in_range_object(&mut self, guid: &Guid) -> Result<()> {
        self.in_range_objects.remove(guid);
        self.recently_removed_guids.push(*guid);
        Ok(())
    }

    fn clear_in_range_objects(&mut self) {
        self.in_range_objects.clear();
    }

    fn get_recently_removed_range_guids(&self) -> &Vec<Guid> {
        &self.recently_removed_guids
    }

    fn clear_recently_removed_range_guids(&mut self) -> Result<()> {
        self.recently_removed_guids.clear();
        Ok(())
    }

    fn wants_updates(&self) -> bool {
        true
    }
}

#[async_trait::async_trait]
impl ReceiveUpdates for Character {
    fn push_update_block(&mut self, data: &mut Vec<u8>, block_count: u32) {
        self.creation_buffer.append(data);
        self.creation_block_count += block_count;
    }
    fn get_update_blocks(&self) -> (u32, &Vec<u8>) {
        (self.creation_block_count, &self.creation_buffer)
    }
    fn clear_update_blocks(&mut self) {
        self.creation_block_count = 0;
        self.creation_buffer.clear();
    }

    async fn process_pending_updates(&mut self) -> Result<()> {
        let (num, buf) = self.get_update_blocks();
        if num > 0 {
            info!("sent {} pending updates to {}", num, self.name);
            //handlers::send_update_packet(self, num, buf).await?;
            self.clear_update_blocks();
        }
        Ok(())
    }
}

impl ValueFieldsRaw for Character {
    fn set_field_u32(&mut self, field: usize, value: u32) -> Result<()> {
        if field > self.unit_value_fields.len() {
            bail!("Out-of-range unit field being set")
        }
        self.unit_value_fields[field] = value;
        self.changed_update_mask.set_bit(field, true)?;
        trace!("Unit field {} on character {} set to {:#08x}", field, self.name, value);
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

    fn clear_update_mask(&mut self) {
        self.changed_update_mask.clear();
    }

    fn get_update_mask(&self) -> &UpdateMask {
        &self.changed_update_mask
    }
}
