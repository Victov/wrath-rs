use std::sync::Weak;

use smol::lock::RwLock;
use wow_world_messages::wrath::{MovementInfo, ObjectType, UpdateMask};
use wow_world_messages::Guid;

use super::map_manager::MapManager;
use super::prelude::ReceiveUpdates;
use crate::character::Character;
use crate::data::PositionAndOrientation;
use crate::prelude::*;

pub trait GameObject: Send + Sync {
    //Gets position of object. Some objects may not have position (Item, Container) = None
    fn get_position(&self) -> Option<PositionAndOrientation>;
    fn get_movement_info(&self) -> &MovementInfo;
    fn get_update_mask(&self) -> UpdateMask;
    fn clear_update_mask_header(&mut self);
    fn is_in_range(&self, guid: Guid) -> bool;
    fn add_in_range_object(&mut self, guid: Guid, object: Weak<RwLock<dyn GameObject>>) -> Result<()>;
    fn add_in_range_character(&mut self, guid: Guid) -> Result<()>;
    fn get_in_range_guids(&self) -> Vec<Guid>;
    fn get_in_range_characters(&self) -> &[Guid];
    fn remove_in_range_object(&mut self, guid: Guid) -> Result<()>;
    fn clear_in_range_objects(&mut self);
    fn get_recently_removed_range_guids(&self) -> &[Guid];
    fn clear_recently_removed_range_guids(&mut self);

    fn as_character(&self) -> Option<&Character>;
    fn as_update_receiver(&self) -> Option<&dyn ReceiveUpdates>;
    fn as_update_receiver_mut(&mut self) -> Option<&mut dyn ReceiveUpdates>;

    fn get_guid(&self) -> Guid;
    fn get_type(&self) -> ObjectType;
    fn on_pushed_to_map(&mut self, map_manager: &MapManager) -> Result<()>;
}
