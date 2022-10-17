use super::world::prelude::*;
use crate::client::Client;
use crate::data::{ActionBar, DataStorage, PositionAndOrientation, TutorialFlags, WorldZoneLocation};
use crate::handlers::login_handler::LogoutState;
use crate::handlers::movement_handler::TeleportationState;
use crate::prelude::*;
use crate::world::prelude::unit_flags::{UnitFlagIndex, UnitFlags};
use async_std::sync::RwLock;
use bit_field::BitField;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};
use wow_dbc::Indexable;
use wow_world_messages::wrath::{
    Area, Class, Gender, Map, MovementInfo, ObjectType, Power, Race, RelationType, UnitStandState, UpdateMask, UpdatePlayer, Vector3d,
};
use wrath_realm_db::RealmDatabase;

mod character_logout;
mod character_movement;
mod character_rested;

pub struct Character {
    pub client: Weak<Client>,
    pub gameplay_data: UpdatePlayer,
    pub name: String,
    pub movement_info: MovementInfo,

    pub map: wow_world_messages::wrath::Map,
    pub area: wow_world_messages::wrath::Area,
    pub instance_id: u32,
    pub bind_location: Option<WorldZoneLocation>,
    pub tutorial_flags: TutorialFlags,
    pub action_bar: ActionBar,

    //Stuff to keep track of playtime
    pub seconds_played_total: u32,
    pub seconds_played_at_level: u32,
    pub last_playtime_calculation_timestamp: u32,

    //required for world updates and implenting ReceiveUpdates trait
    pending_object_updates: Vec<wow_world_messages::wrath::Object>,

    //things required make GameObject working
    in_range_objects: HashMap<Guid, Weak<RwLock<dyn GameObject>>>,
    recently_removed_guids: Vec<Guid>,

    //time sync
    pub time_sync_counter: u32,
    time_sync_cooldown: f32,

    //Teleporting
    pub teleportation_state: TeleportationState,
    pub logout_state: LogoutState,
    rested_state: character_rested::RestedState,
}

impl Character {
    pub fn new(client: Weak<Client>, guid: Guid) -> Self {
        Self {
            client,
            gameplay_data: UpdatePlayer::builder().set_object_GUID(guid).finalize(),
            name: String::new(),
            movement_info: MovementInfo::default(),
            map: Map::EasternKingdoms,
            area: Area::NorthshireAbbey,
            instance_id: 0,
            bind_location: None,
            tutorial_flags: TutorialFlags::default(),
            action_bar: ActionBar::new(),
            seconds_played_total: 0,
            seconds_played_at_level: 0,
            last_playtime_calculation_timestamp: 0,
            pending_object_updates: vec![],
            in_range_objects: HashMap::new(),
            recently_removed_guids: vec![],
            time_sync_counter: 0,
            time_sync_cooldown: 0f32,
            teleportation_state: TeleportationState::None,
            logout_state: LogoutState::None,
            rested_state: character_rested::RestedState::NotRested,
        }
    }

    pub async fn load_from_database(&mut self, world: &World, data_storage: &DataStorage) -> Result<()> {
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

        Ok(())
    }

    pub async fn send_packets_before_add_to_map(&self) -> Result<()> {
        handlers::send_contact_list(self, RelationType::empty().set_FRIEND().set_IGNORED().set_MUTED().set_RECRUITAFRIEND()).await?;
        handlers::send_bind_update(self).await?;
        handlers::send_dungeon_difficulty(self).await?;
        handlers::send_action_buttons(self).await?;
        handlers::send_initial_world_states(self).await?;
        handlers::send_login_set_time_speed(self).await
    }

    pub async fn send_packets_after_add_to_map(&self, realm_database: Arc<RealmDatabase>) -> Result<()> {
        handlers::send_verify_world(self).await?;
        handlers::send_character_account_data_times(&realm_database, self).await?;
        handlers::send_voice_chat_status(self).await?;
        handlers::send_tutorial_flags(self).await?;
        handlers::send_faction_list(self).await?;
        handlers::send_time_sync(self).await?;
        Ok(())
    }

    pub async fn zone_update(&mut self, area: Area) -> Result<()> {
        if self.area == area {
            return Ok(());
        }

        trace!("Received zone update for character {} into zone {}", self.name, area);
        self.area = area;
        handlers::send_initial_world_states(self).await
    }

    pub fn reset_time_sync(&mut self) {
        self.time_sync_cooldown = 0.0;
        self.time_sync_counter = 0;
    }

    pub async fn tick(&mut self, delta_time: f32, world: Arc<World>) -> Result<()> {
        self.tick_time_sync(delta_time).await?;
        self.tick_logout_state(delta_time, world.clone()).await?;

        self.handle_queued_teleport(world)
            .await
            .unwrap_or_else(|e| warn!("Could not teleport player {}: Error {}", self.name, e));

        Ok(())
    }
    async fn tick_time_sync(&mut self, delta_time: f32) -> Result<()> {
        self.time_sync_cooldown -= delta_time;
        if self.time_sync_cooldown < 0f32 {
            self.time_sync_cooldown += 10f32;
            self.time_sync_counter = self.time_sync_counter.wrapping_add(1);
            handlers::send_time_sync(self).await?;
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

    pub fn set_selection(&mut self, new_selection: Option<Guid>) {
        let guid = new_selection.unwrap_or_else(Guid::zero);
        self.gameplay_data.set_unit_TARGET(guid);
    }

    pub fn get_selection(&self) -> Option<Guid> {
        self.gameplay_data.unit_TARGET().and_then(|g| if g.is_zero() { None } else { Some(g) })
    }

    //-------------------
    //BEGIN STUFF THAT NEEDS TO MOVE TO UpdateMaskExt
    //-------------------
    async fn set_stand_state(&mut self, state: UnitStandState) -> Result<()> {
        let (_, b, c, d) = self.gameplay_data.unit_BYTES_1().unwrap_or_default();
        self.gameplay_data.set_unit_BYTES_1(state, b, c, d);
        handlers::send_smsg_stand_state_update(self, state).await
    }

    fn set_unit_flag_byte(&mut self, unit_flag: UnitFlagIndex, value: bool) {
        let mut unit_flags: i32 = self.gameplay_data.unit_FLAGS().unwrap_or(0);
        unit_flags.set_bit(unit_flag as usize, value);
        self.gameplay_data.set_unit_FLAGS(unit_flags);
    }

    async fn set_rooted(&self, rooted: bool) -> Result<()> {
        if rooted {
            handlers::send_smsg_force_move_root(self).await
        } else {
            handlers::send_smsg_force_move_unroot(self).await
        }
    }

    fn set_stunned(&mut self, stunned: bool) {
        self.set_unit_flag_byte(UnitFlagIndex::Stunned, stunned)
    }

    fn set_rested_bytes(&mut self, rested: bool) -> Result<()> {
        let _value = match rested {
            true => 1,
            false => 2,
        };
        //self.set_byte(PlayerFields::Bytes2 as usize, 3, value)
        Ok(())
    }

    pub fn get_race(&self) -> Race {
        self.gameplay_data.unit_BYTES_0().map_or(Race::Human, |(race, _, _, _)| race)
    }

    pub fn get_class(&self) -> Class {
        self.gameplay_data.unit_BYTES_0().map_or(Class::Warrior, |(_, class, _, _)| class)
    }

    pub fn get_gender(&self) -> Gender {
        self.gameplay_data.unit_BYTES_0().map_or(Gender::Male, |(_, _, gender, _)| gender)
    }

    pub fn get_power_type(&self) -> Power {
        self.gameplay_data.unit_BYTES_0().map_or(Power::Mana, |(_, _, _, power)| power)
    }

    //-------------------
    //END STUFF THAT NEEDS TO MOVE TO UpdateMaskExt
    //-------------------
}

#[async_trait::async_trait]
impl GameObject for Character {
    fn get_guid(&self) -> Guid {
        self.gameplay_data.object_GUID().unwrap()
    }

    fn get_type(&self) -> ObjectType {
        ObjectType::Player
    }

    fn get_update_mask(&self) -> UpdateMask {
        UpdateMask::Player(self.gameplay_data.clone())
    }

    fn clear_update_mask_header(&mut self) {
        self.gameplay_data.dirty_reset();
    }

    async fn on_pushed_to_map(&mut self, _map_manager: &MapManager) -> Result<()> {
        let create_block = build_create_update_block_for_player(self, self)?;
        self.push_object_update(create_block);
        self.process_pending_updates().await
    }

    fn as_character(&self) -> Option<&Character> {
        Some(self)
    }

    fn get_position(&self) -> Option<PositionAndOrientation> {
        Some(PositionAndOrientation {
            position: self.movement_info.position,
            orientation: self.movement_info.orientation,
        })
    }

    fn get_movement_info(&self) -> &MovementInfo {
        &self.movement_info
    }

    fn is_in_range(&self, guid: Guid) -> bool {
        self.in_range_objects.contains_key(&guid)
    }

    fn add_in_range_object(&mut self, guid: Guid, object: Weak<RwLock<dyn GameObject>>) -> Result<()> {
        assert!(!self.is_in_range(guid));
        self.in_range_objects.insert(guid, object);
        Ok(())
    }

    fn get_in_range_guids(&self) -> Vec<Guid> {
        self.in_range_objects.keys().copied().collect()
    }

    fn remove_in_range_object(&mut self, guid: Guid) -> Result<()> {
        self.in_range_objects.remove(&guid);
        self.recently_removed_guids.push(guid);
        Ok(())
    }

    fn clear_in_range_objects(&mut self) {
        self.in_range_objects.clear();
    }

    fn get_recently_removed_range_guids(&self) -> &[Guid] {
        self.recently_removed_guids.as_slice()
    }

    fn clear_recently_removed_range_guids(&mut self) {
        self.recently_removed_guids.clear();
    }

    fn as_update_receiver(&self) -> Option<&dyn ReceiveUpdates> {
        Some(self)
    }

    fn as_update_receiver_mut(&mut self) -> Option<&mut dyn ReceiveUpdates> {
        Some(self)
    }
}

#[async_trait::async_trait]
impl ReceiveUpdates for Character {
    fn push_object_update(&mut self, object_update: wow_world_messages::wrath::Object) {
        self.pending_object_updates.push(object_update);
    }

    fn get_object_updates(&self) -> &Vec<wow_world_messages::wrath::Object> {
        &self.pending_object_updates
    }

    fn clear_object_updates(&mut self) {
        self.pending_object_updates.clear();
    }

    async fn process_pending_updates(&mut self) -> Result<()> {
        let updates = self.get_object_updates();
        if !updates.is_empty() {
            handlers::send_smsg_update_objects(self, updates.clone()).await?;
            self.clear_object_updates();
        }
        Ok(())
    }
}
