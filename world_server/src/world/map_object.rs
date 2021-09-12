use crate::{constants::updates::ObjectType, data_types::PositionAndOrientation, guid::Guid};

pub use super::map_cell::MapCell;

pub trait MapObject: PartialEq {
    fn get_guid(&self) -> &Guid;
    fn get_position(&self) -> PositionAndOrientation;
    fn set_in_cell(&mut self, cell: &MapCell);
    fn get_type(&self) -> ObjectType;
}
