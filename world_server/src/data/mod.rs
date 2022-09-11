use std::fmt::Display;

use anyhow::Result;

mod tutorial_flags;
pub use tutorial_flags::TutorialFlags;

mod packed_time;
pub use packed_time::PackedTime;

mod action_bar;
pub use action_bar::ActionBar;

mod movement_info;
pub use movement_info::*;

pub mod guid;

mod data_storage;
pub use data_storage::*;

#[derive(PartialEq, Debug, Default, Clone)]
pub struct WorldZoneLocation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub o: f32,
    pub zone: u32,
    pub map: u32,
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct PositionAndOrientation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub o: f32,
}

impl Display for PositionAndOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x{}, y{}, z{}, o{})", self.x, self.y, self.z, self.o)
    }
}

impl From<WorldZoneLocation> for PositionAndOrientation {
    fn from(wzl: WorldZoneLocation) -> Self {
        Self {
            x: wzl.x,
            y: wzl.y,
            z: wzl.z,
            o: wzl.o,
        }
    }
}

pub trait ReadPositionAndOrientation {
    fn read_position_and_orientation(&mut self) -> Result<PositionAndOrientation>;
}

impl<R: std::io::Read> ReadPositionAndOrientation for R {
    fn read_position_and_orientation(&mut self) -> Result<PositionAndOrientation> {
        use podio::{LittleEndian, ReadPodExt};

        let x = self.read_f32::<LittleEndian>()?;
        let y = self.read_f32::<LittleEndian>()?;
        let z = self.read_f32::<LittleEndian>()?;
        let o = self.read_f32::<LittleEndian>()?;
        Ok(PositionAndOrientation { x, y, z, o })
    }
}

pub trait WritePositionAndOrientation {
    fn write_position_and_orientation(&mut self, position: &PositionAndOrientation) -> Result<()>;
}

impl<W: std::io::Write> WritePositionAndOrientation for W {
    fn write_position_and_orientation(&mut self, position: &PositionAndOrientation) -> Result<()> {
        use podio::{LittleEndian, WritePodExt};
        self.write_f32::<LittleEndian>(position.x)?;
        self.write_f32::<LittleEndian>(position.y)?;
        self.write_f32::<LittleEndian>(position.z)?;
        self.write_f32::<LittleEndian>(position.o)?;
        Ok(())
    }
}
