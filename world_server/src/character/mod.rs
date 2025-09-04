use self::character_inventory::{BagInventory, GameplayCharacterInventory};

use super::world::prelude::*;
use crate::connection::events::ServerEvent;
use crate::data::{ActionBar, DataStorage, PositionAndOrientation, TutorialFlags, WorldZoneLocation};
use crate::handlers::login_handler::LogoutState;
use crate::handlers::movement_handler::TeleportationState;
use crate::prelude::*;
use crate::world::prelude::unit_flags::UnitFlagIndex;
use bit_field::BitField;
use smol::lock::RwLock;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use wow_world_messages::wrath::{
    ActionButton, Area, Class, Gender, Map, MovementInfo, ObjectType, Power, Race, RelationType, UnitStandState, UpdateMask, UpdatePlayer,
};
use wrath_realm_db::RealmDatabase;

mod character_cinematic;
mod character_database;
mod character_first_login;
pub mod character_inventory;
mod character_logout;
pub mod character_manager;
mod character_movement;
mod character_rested;

pub struct Character {
    // Both client and character have a sender to the connection
    pub connection_sender: flume::Sender<ServerEvent>,

    /// This sender is cloned an used to send world updates to the character
    pub sender: flume::Sender<wow_world_messages::wrath::Object>,
    /// This receives world updates that are sent to the character
    pub receiver: flume::Receiver<wow_world_messages::wrath::Object>,

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
    in_range_characters: Vec<Guid>,
    recently_removed_guids: Vec<Guid>,

    //time sync
    pub time_sync_counter: u32,
    time_sync_cooldown: f32,

    //Teleporting
    pub teleportation_state: TeleportationState,
    pub logout_state: LogoutState,
    rested_state: character_rested::RestedState,

    //Very first login
    needs_first_login: bool,

    cinematic_state: character_cinematic::CharacterCinematicState,

    //items
    pub equipped_items: GameplayCharacterInventory,
    pub bag_items: BagInventory,
}

impl Character {
    pub fn new(connection_sender: flume::Sender<ServerEvent>, guid: Guid) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            connection_sender,
            sender,
            receiver,
            gameplay_data: UpdatePlayer::builder().set_object_guid(guid).finalize(),
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
            in_range_characters: vec![],
            recently_removed_guids: vec![],
            time_sync_counter: 0,
            time_sync_cooldown: 0f32,
            teleportation_state: TeleportationState::None,
            logout_state: LogoutState::None,
            rested_state: character_rested::RestedState::NotRested,
            needs_first_login: false,
            cinematic_state: character_cinematic::CharacterCinematicState::None,
            equipped_items: GameplayCharacterInventory::new(),
            bag_items: BagInventory::default(),
        }
    }

    pub fn get_sender(&self) -> flume::Sender<wow_world_messages::wrath::Object> {
        self.sender.clone()
    }

    pub async fn load(connection_sender: flume::Sender<ServerEvent>, guid: Guid, world: &World, data_storage: &DataStorage) -> Result<Self> {
        let mut character = Self::new(connection_sender, guid);
        character.load_from_database_internal(world, data_storage).await?;
        Ok(character)
    }

    pub async fn send_packets_before_add_to_map(&self) -> Result<()> {
        handlers::send_contact_list(self, RelationType::empty().set_friend().set_ignored().set_muted().set_recruitafriend()).await?;
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

    pub async fn tick(&mut self, delta_time: f32, world: &mut World) -> Result<()> {
        self.try_perform_first_time_login_if_required().await?;
        self.tick_time_sync(delta_time).await?;
        self.tick_logout_state(delta_time, world).await?;

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

    //pub async fn try_get_self_arc(&self) -> Result<Arc<RwLock<Self>>> {
    //    let client = self
    //        .client
    //        .upgrade()
    //        .ok_or_else(|| anyhow!("Could not get an Arc to the character because the owning client does not exist"))?;
    //    client.get_active_character().await
    //}

    pub fn set_selection(&mut self, new_selection: Option<Guid>) {
        let guid = new_selection.unwrap_or_else(Guid::zero);
        self.gameplay_data.set_unit_target(guid);
    }

    pub fn get_selection(&self) -> Option<Guid> {
        self.gameplay_data.unit_target().and_then(|g| if g.is_zero() { None } else { Some(g) })
    }

    //-------------------
    //BEGIN STUFF THAT NEEDS TO MOVE TO UpdateMaskExt
    //-------------------
    pub async fn set_stand_state(&mut self, state: UnitStandState) -> Result<()> {
        let (_, b, c, d) = self.gameplay_data.unit_bytes_1().unwrap_or_default();
        self.gameplay_data.set_unit_bytes_1(state, b, c, d);
        handlers::send_smsg_stand_state_update(self, state).await
    }

    fn set_unit_flag_byte(&mut self, unit_flag: UnitFlagIndex, value: bool) {
        let mut unit_flags: i32 = self.gameplay_data.unit_flags().unwrap_or(0);
        unit_flags.set_bit(unit_flag as usize, value);
        self.gameplay_data.set_unit_flags(unit_flags);
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

    pub fn set_visible_actionbar_mask(&mut self, action_bars: u8) {
        let (a, b, _, d) = self.gameplay_data.player_features().unwrap_or_default();
        //action_bars is a flags, no extra actionbars = 0, all bars (2 above default bar, 2 side
        //bars) is 15 when they are set visible in the Interface settings menu
        self.gameplay_data.set_player_features(a, b, action_bars, d);
    }

    pub fn set_action_bar_button(&mut self, slot: u8, action_button: ActionButton) {
        self.action_bar.set_action_button(slot as usize, action_button);
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
        self.gameplay_data.unit_bytes_0().map_or(Race::Human, |(race, _, _, _)| race)
    }

    pub fn get_class(&self) -> Class {
        self.gameplay_data.unit_bytes_0().map_or(Class::Warrior, |(_, class, _, _)| class)
    }

    pub fn get_gender(&self) -> Gender {
        self.gameplay_data.unit_bytes_0().map_or(Gender::Male, |(_, _, gender, _)| gender)
    }

    pub fn get_power_type(&self) -> Power {
        self.gameplay_data.unit_bytes_0().map_or(Power::Mana, |(_, _, _, power)| power)
    }

    //-------------------
    //END STUFF THAT NEEDS TO MOVE TO UpdateMaskExt
    //-------------------
}

#[async_trait::async_trait]
impl GameObject for Character {
    fn get_guid(&self) -> Guid {
        self.gameplay_data.object_guid().unwrap()
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

    fn on_pushed_to_map(&mut self, _map_manager: &MapManager) -> Result<()> {
        let create_block = build_create_update_block_for_player(self, self)?;
        self.push_object_update(create_block);
        Ok(())
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

    fn add_in_range_character(&mut self, guid: Guid) -> Result<()> {
        if !self.in_range_characters.contains(&guid) {
            self.in_range_characters.push(guid);
        }
        Ok(())
    }

    fn get_in_range_guids(&self) -> Vec<Guid> {
        self.in_range_objects.keys().copied().collect()
    }

    fn get_in_range_characters(&self) -> &[Guid] {
        &self.in_range_characters
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
