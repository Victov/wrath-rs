mod tutorial_flags;
pub use tutorial_flags::TutorialFlags;

mod action_bar;
pub use action_bar::ActionBar;

mod data_storage;
pub use data_storage::*;

use wow_world_messages::wrath::{Area, Map, Vector3d};
pub struct WorldZoneLocation {
    pub map: Map,
    pub area: Area,
    pub position: Vector3d,
    pub orientation: f32,
}

pub struct PositionAndOrientation {
    pub position: Vector3d,
    pub orientation: f32,
}
