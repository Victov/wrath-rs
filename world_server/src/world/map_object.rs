use std::sync::Weak;

use async_std::sync::RwLock;
use wow_world_messages::wrath::MovementInfo;
use wow_world_messages::Guid;

pub use super::map_cell::MapCell;
use super::map_manager::MapManager;
//use super::update_builder::ReceiveUpdates;
use super::value_fields::HasValueFields;
use crate::character::Character;
use crate::constants::updates::ObjectType;
use crate::data::PositionAndOrientation;
use crate::prelude::*;

pub trait GameObject: Sync + Send {
    fn as_world_object(&self) -> Option<&dyn WorldObject>;
    fn as_world_object_mut(&mut self) -> Option<&mut dyn WorldObject>;
    fn as_map_object(&self) -> &dyn MapObject; //Mandatory to implement this!
    fn as_map_object_mut(&mut self) -> &mut dyn MapObject; //Mandatory to implement
    fn as_has_value_fields(&self) -> Option<&dyn HasValueFields>;
    fn as_has_value_fields_mut(&mut self) -> Option<&mut dyn HasValueFields>;
    //fn as_update_receiver(&self) -> Option<&dyn ReceiveUpdates>;
    //fn as_update_receiver_mut(&mut self) -> Option<&mut dyn ReceiveUpdates>;

    //Special case shortcut
    fn as_character(&self) -> Option<&Character>;
}

pub trait WorldObject: MapObject {
    fn get_position(&self) -> PositionAndOrientation;
    fn get_movement_info(&self) -> &MovementInfo;
    fn is_in_range(&self, guid: Guid) -> bool;
    fn add_in_range_object(&mut self, guid: Guid, object: Weak<RwLock<dyn GameObject>>) -> Result<()>;
    fn get_in_range_guids(&self) -> Vec<Guid>;
    fn remove_in_range_object(&mut self, guid: Guid) -> Result<()>;
    fn clear_in_range_objects(&mut self);
    fn get_recently_removed_range_guids(&self) -> &[Guid];
    fn clear_recently_removed_range_guids(&mut self);
    fn wants_updates(&self) -> bool;

    fn is_player(&self) -> bool {
        self.get_type() as u8 & ObjectType::Player as u8 > 0
    }
}

#[async_trait::async_trait]
pub trait MapObject: Sync + Send {
    fn get_guid(&self) -> Guid;
    fn get_type(&self) -> ObjectType;
    async fn on_pushed_to_map(&mut self, map_manager: &MapManager) -> Result<()>;
}
