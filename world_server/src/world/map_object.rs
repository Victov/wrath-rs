use std::sync::Weak;

use async_std::sync::RwLock;

pub use super::map_cell::MapCell;
use super::map_manager::MapManager;
use super::update_builder::{MapObjectWithValueFields, ReceiveUpdates};
use crate::character::Character;
use crate::prelude::*;
use crate::{constants::updates::ObjectType, data::PositionAndOrientation};

pub trait MapObject: Send + Sync {
    fn get_guid(&self) -> &Guid;
    fn get_position(&self) -> &PositionAndOrientation;
    fn get_type(&self) -> ObjectType;

    fn on_pushed_to_map(&mut self, map_manager: &MapManager) -> Result<()>;
    fn is_in_range(&self, guid: &Guid) -> bool;
    fn add_in_range_object(&mut self, guid: &Guid, object: Weak<RwLock<dyn MapObjectWithValueFields>>) -> Result<()>;
    fn get_in_range_guids(&self) -> Vec<&Guid>;
    fn remove_in_range_object(&mut self, guid: &Guid) -> Result<()>;
    fn get_recently_removed_range_guids(&self) -> &Vec<Guid>;
    fn clear_recently_removed_range_guids(&mut self) -> Result<()>;
    fn wants_updates(&self) -> bool;
    fn as_update_receiver_mut(&mut self) -> Option<&mut dyn ReceiveUpdates>;
    fn as_character(&self) -> Option<&Character>;

    fn is_player(&self) -> bool {
        self.get_type() as u8 & ObjectType::Player as u8 > 0
    }
}
