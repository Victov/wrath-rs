use anyhow::Result;

mod tutorial_flags;
pub use tutorial_flags::TutorialFlags;

mod packed_time;
pub use packed_time::{PackedTime, ReadPackedTime, WritePackedTime};

mod action_bar;
pub use action_bar::ActionBar;

mod movement_info;
pub use movement_info::*;

pub mod guid;

pub struct WorldZoneLocation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub zone: u32,
    pub map: u32,
}

#[derive(Default)]
pub struct PositionAndOrientation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub o: f32,
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
