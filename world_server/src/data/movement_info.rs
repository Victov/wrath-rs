use crate::data::{ReadPositionAndOrientation, WritePositionAndOrientation};

use super::PositionAndOrientation;
use anyhow::Result;

#[derive(Default, Clone)]
pub struct MovementInfo {
    pub movement_flags: u32,
    pub movement_flags2: u16,
    pub time: u32,
    pub position: PositionAndOrientation,
    pub falltime: u32,
}

pub trait ReadMovementInfo {
    fn read_movement_info(&mut self) -> Result<MovementInfo>;
}

impl<R: std::io::Read> ReadMovementInfo for R {
    fn read_movement_info(&mut self) -> Result<MovementInfo> {
        use podio::{LittleEndian, ReadPodExt};

        let mut info = MovementInfo::default();
        info.movement_flags = self.read_u32::<LittleEndian>()?;
        info.movement_flags2 = self.read_u16::<LittleEndian>()?;
        info.time = self.read_u32::<LittleEndian>()?;
        info.position = self.read_position_and_orientation()?;
        info.falltime = self.read_u32::<LittleEndian>()?;

        Ok(info)
    }
}

pub trait WriteMovementInfo {
    fn write_movement_info(&mut self, info: &MovementInfo) -> Result<()>;
}

impl<W: std::io::Write> WriteMovementInfo for W {
    fn write_movement_info(&mut self, info: &MovementInfo) -> Result<()> {
        use podio::{LittleEndian, WritePodExt};
        self.write_u32::<LittleEndian>(info.movement_flags)?;
        self.write_u16::<LittleEndian>(info.movement_flags2)?;
        self.write_u32::<LittleEndian>(info.time)?;
        self.write_position_and_orientation(&info.position)?;
        self.write_u32::<LittleEndian>(info.falltime)?;
        Ok(())
    }
}
