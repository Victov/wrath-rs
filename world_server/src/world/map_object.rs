pub use super::map_cell::MapCell;
use crate::{constants::updates::ObjectType, data_types::PositionAndOrientation, guid::Guid};

pub trait MapObject: Send + Sync {
    fn get_guid(&self) -> &Guid;
    fn get_position(&self) -> PositionAndOrientation;
    fn set_in_cell(&mut self, cell: &MapCell);
    fn get_type(&self) -> ObjectType;

    fn is_player(&self) -> bool {
        self.get_type() as u8 & ObjectType::Player as u8 > 0
    }
}
