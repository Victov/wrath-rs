mod tutorial_flags;
pub use tutorial_flags::TutorialFlags;

mod packed_time;
pub use packed_time::{WritePackedTime, ReadPackedTime, PackedTime};

pub struct WorldZoneLocation
{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub zone: u32,
    pub map: u32
}
