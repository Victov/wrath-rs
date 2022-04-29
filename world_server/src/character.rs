use super::world::prelude::*;
use crate::client::Client;
use crate::constants::social::RelationType;
use crate::data::DBCStorage;
use crate::data::{ActionBar, MovementFlags, MovementInfo, PositionAndOrientation, TutorialFlags, WorldZoneLocation};
use crate::handlers::login_handler::{LogoutResult, LogoutSpeed};
use crate::handlers::{
    login_handler::LogoutState,
    movement_handler::{TeleportationDistance, TeleportationState},
};
use crate::prelude::*;
use crate::ClientManager;
use async_std::sync::RwLock;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};
use wrath_realm_db::RealmDatabase;

const NUM_UNIT_FIELDS: usize = PlayerFields::PlayerEnd as usize;

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
    in_range_objects: HashMap<Guid, Weak<RwLock<dyn MapObjectWithValueFields>>>,
    recently_removed_guids: Vec<Guid>,

    //time sync
    pub time_sync_counter: u32,
    time_sync_cooldown: f32,

    //Teleporting
    pub teleportation_state: TeleportationState,

    pub logout_state: LogoutState,
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
            teleportation_state: TeleportationState::None,
            logout_state: LogoutState::None,
        }
    }

    pub async fn load_from_database(&mut self, dbc_storage: &DBCStorage, realm_database: &RealmDatabase) -> Result<()> {
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
            handlers::create_empty_character_account_data_rows(realm_database, character_id).await?;
        }

        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

        self.last_playtime_calculation_timestamp = unix_time;
        self.seconds_played_total = db_entry.playtime_total;
        self.seconds_played_at_level = db_entry.playtime_level;

        self.gender = db_entry.gender;
        self.race = db_entry.race;
        if let Some(race_info) = dbc_storage.get_dbc_char_races()?.get_entry(self.race as u32) {
            let display_id = match self.gender {
                0 => race_info.male_model_id,
                _ => race_info.female_model_id,
            };
            self.set_unit_field_u32(UnitFields::Displayid, display_id)?;
            self.set_unit_field_u32(UnitFields::Nativedisplayid, display_id)?;
        }

        self.class = db_entry.class;
        if let Some(class_info) = dbc_storage.get_dbc_char_classes()?.get_entry(self.class as u32) {
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

        Ok(())
    }

    pub async fn send_packets_before_add_to_map(&self, _client_manager: &ClientManager) -> Result<()> {
        handlers::send_contact_list(self, &[RelationType::Friend, RelationType::Muted, RelationType::Ignore]).await?;
        handlers::send_bind_update(self).await?;
        handlers::send_talents_info(self).await?;
        handlers::send_dungeon_difficulty(self).await?;
        handlers::send_initial_spells(self).await?;
        handlers::send_action_buttons(self).await?;
        handlers::send_initial_world_states(self).await?;
        handlers::send_login_set_time_speed(self).await
    }

    pub async fn send_packets_after_add_to_map(&self, client_manager: &ClientManager) -> Result<()> {
        handlers::send_verify_world(self).await?;
        handlers::send_character_account_data_times(client_manager, self).await?;
        handlers::send_voice_chat_status(self, false).await?;
        handlers::send_tutorial_flags(self).await?;
        handlers::send_faction_list(self).await?;
        handlers::send_aura_update_all(self).await?;
        handlers::send_time_sync(self).await?;
        //handlers::send_world_state_update(&self, 0xF3D, 0).await?;
        //handlers::send_world_state_update(&self, 0xC77, 0).await?;

        Ok(())
    }

    pub async fn zone_update(&mut self, zone: u32) -> Result<()> {
        if self.zone == zone {
            return Ok(());
        }

        trace!("Received zone update for character {} into zone {}", self.name, zone);
        self.zone = zone;
        handlers::send_initial_world_states(self).await
    }

    pub fn reset_time_sync(&mut self) {
        self.time_sync_cooldown = 0.0;
        self.time_sync_counter = 0;
    }

    pub fn process_movement(&mut self, movement_info: &MovementInfo) {
        self.movement_info = movement_info.clone();
    }

    pub fn set_position(&mut self, position: &PositionAndOrientation) {
        self.movement_info.position = position.clone();
    }

    pub fn teleport_to(&mut self, destination: TeleportationDistance) {
        self.teleportation_state = TeleportationState::Queued(destination);
    }

    pub async fn tick(&mut self, delta_time: f32, world: Arc<World>) -> Result<()> {
        self.tick_time_sync(delta_time).await?;
        self.tick_logout_state(delta_time, world.clone()).await?;

        self.handle_queued_teleport(world)
            .await
            .unwrap_or_else(|e| warn!("Could not teleport player {}: Error {}", self.name, e));

        Ok(())
    }

    async fn handle_queued_teleport(&mut self, world: Arc<World>) -> Result<()> {
        //Handle the possibility that the player may have logged out
        //between queuing and handling the teleport

        let state = self.teleportation_state.clone();
        match state {
            TeleportationState::Queued(TeleportationDistance::Near(dest)) => self.execute_near_teleport(dest.clone()).await?,
            TeleportationState::Queued(TeleportationDistance::Far(dest)) => self.execute_far_teleport(dest.clone(), world).await?,
            _ => {}
        };

        Ok(())
    }

    async fn tick_time_sync(&mut self, delta_time: f32) -> Result<()> {
        self.time_sync_cooldown -= delta_time;
        if self.time_sync_cooldown < 0f32 {
            self.time_sync_cooldown += 10f32;
            self.time_sync_counter += 1;
            handlers::send_time_sync(self).await?;
        }
        Ok(())
    }

    async fn tick_logout_state(&mut self, delta_time: f32, world: Arc<World>) -> Result<()> {
        match &mut self.logout_state {
            LogoutState::Pending(duration_left) => {
                *duration_left = duration_left.saturating_sub(std::time::Duration::from_secs_f32(delta_time));
                if duration_left.is_zero() {
                    self.logout_state = LogoutState::Executing;
                }
                Ok(())
            }
            LogoutState::Executing => self.execute_logout(world).await,
            _ => Ok(()),
        }
    }

    async fn execute_near_teleport(&mut self, destination: PositionAndOrientation) -> Result<()> {
        //The rest of the teleportation is handled when the client sends back this packet

        self.teleportation_state = TeleportationState::Executing(TeleportationDistance::Near(destination.clone()));

        handlers::send_msg_move_teleport_ack(self, &destination).await?;
        Ok(())
    }

    async fn execute_far_teleport(&mut self, destination: WorldZoneLocation, world: Arc<World>) -> Result<()> {
        if self.map == destination.map {
            //This was not actually a far teleport. It should have been a near teleport since we're
            //on the same map.
            self.teleport_to(TeleportationDistance::Near(destination.into()));
            return Ok(());
        }

        handlers::send_smsg_transfer_pending(self, destination.map).await?;

        let old_map = world
            .get_instance_manager()
            .try_get_map_for_character(self)
            .await
            .ok_or_else(|| anyhow!("Player is teleporting away from an invalid map"))?;

        old_map.remove_object_by_guid(&self.guid).await;

        let wzl = destination.clone().into();
        handlers::send_smsg_new_world(self, destination.map, &wzl).await?;

        self.teleportation_state = TeleportationState::Executing(TeleportationDistance::Far(destination));
        Ok(())
    }

    pub async fn try_logout(&mut self) -> Result<(LogoutResult, LogoutSpeed)> {
        //TODO: add checks about being in rested area (instant logout), being in combat (refuse), etc

        if self
            .movement_info
            .has_any_movement_flag(&[MovementFlags::Falling, MovementFlags::FallingFar])
        {
            return Ok((LogoutResult::FailJumpingOrFalling, LogoutSpeed::Delayed));
        }

        let delayed = true;
        Ok(match self.logout_state {
            LogoutState::None if delayed => {
                self.logout_state = LogoutState::Pending(std::time::Duration::from_secs(20));
                self.set_stunned(true)?;
                self.set_rooted(true).await?;
                self.set_character_stand_state(stand_state::UnitStandState::Sit).await?;
                (LogoutResult::Success, LogoutSpeed::Delayed)
            }
            LogoutState::None if !delayed => {
                self.logout_state = LogoutState::Executing;
                (LogoutResult::Success, LogoutSpeed::Instant)
            }
            _ => (LogoutResult::Success, LogoutSpeed::Instant),
        })
    }

    pub async fn cancel_logout(&mut self) -> Result<()> {
        if let LogoutState::Pending(_) = self.logout_state {
            self.set_stunned(false)?;
            self.set_rooted(false).await?;
            self.set_character_stand_state(stand_state::UnitStandState::Stand).await?;
            self.logout_state = LogoutState::None;
        } else {
        }
        Ok(())
    }

    //This function will trigger every tick as long as the state is LogoutState::Executing
    async fn execute_logout(&mut self, world: Arc<World>) -> Result<()> {
        if self.teleportation_state != TeleportationState::None {
            return Ok(());
        }

        world
            .get_instance_manager()
            .try_get_map_for_character(self)
            .await
            .ok_or_else(|| anyhow!("Invalid map during logout"))?
            .remove_object_by_guid(&self.guid)
            .await;

        handlers::send_smsg_logout_complete(self).await?;

        self.logout_state = LogoutState::ReturnToCharSelect;

        Ok(())
    }
}

impl MapObject for Character {
    fn get_guid(&self) -> &Guid {
        &self.guid
    }

    fn get_position(&self) -> &PositionAndOrientation {
        &self.movement_info.position
    }

    fn get_movement_info(&self) -> &MovementInfo {
        &self.movement_info
    }

    fn get_type(&self) -> updates::ObjectType {
        ObjectType::Player
    }

    fn on_pushed_to_map(&mut self, _map_manager: &MapManager) -> Result<()> {
        let (block_count, mut update_data) = build_create_update_block_for_player(self, self)?;
        self.push_update_block(&mut update_data, block_count);
        Ok(())
    }

    fn is_in_range(&self, guid: &Guid) -> bool {
        self.in_range_objects.contains_key(guid)
    }

    fn add_in_range_object(&mut self, guid: &Guid, object: Weak<RwLock<dyn MapObjectWithValueFields>>) -> Result<()> {
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

    fn as_update_receiver_mut(&mut self) -> Option<&mut dyn ReceiveUpdates> {
        Some(self)
    }

    fn as_character(&self) -> Option<&Character> {
        Some(self)
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
            handlers::send_update_packet(self, num, buf).await?;
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
