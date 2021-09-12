mod tutorial_flags;
pub use tutorial_flags::TutorialFlags;

mod packed_time;
pub use packed_time::{PackedTime, ReadPackedTime, WritePackedTime};

mod action_bar;
pub use action_bar::ActionBar;

pub struct WorldZoneLocation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub zone: u32,
    pub map: u32,
}

pub struct PositionAndOrientation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub o: f32,
}
