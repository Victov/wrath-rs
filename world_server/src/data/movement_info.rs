use crate::data::{ReadPositionAndOrientation, WritePositionAndOrientation};

use super::PositionAndOrientation;
use anyhow::Result;
use num_enum::TryFromPrimitive;

#[derive(Clone, Copy, Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum MovementFlags {
    None = 0x00000000,
    Forward = 0x00000001,
    Backward = 0x00000002,
    StrafeLeft = 0x00000004,
    StrafeRight = 0x00000008,
    TurnLeft = 0x00000010,
    TurnRight = 0x00000020,
    PitchRp = 0x00000040,
    PitchDown = 0x00000080,

    //MovementFlags 2 from here
    WalkMode = 0x00000100,
    OnTransport = 0x00000200,
    Levitating = 0x00000400,
    Root = 0x00000800,
    Falling = 0x00001000,
    FallingFar = 0x00002000,
    PendingStop = 0x00004000,
    PendingStrafeStop = 0x00008000,
    PendingForward = 0x00010000,
    PendingBackward = 0x00020000,
    PendingStrafeLeft = 0x00040000,
    PendingStrafeRight = 0x00080000,
    PendingRoot = 0x00100000,
    Swimming = 0x00200000,
    Ascending = 0x00400000,
    Descending = 0x00800000,
    CanFly = 0x01000000,
    Flying = 0x02000000,
    SplineElevation = 0x04000000,
    SplineEnabled = 0x08000000,
    WaterWalking = 0x10000000,
    SafeFall = 0x20000000,
    Hover = 0x40000000,
}

#[derive(Default, Clone)]
pub struct MovementInfo {
    pub movement_flags: u32,
    pub movement_flags2: u16,
    pub time: u32,
    pub position: PositionAndOrientation,
    pub falltime: u32,
}

impl MovementInfo {
    pub fn has_movement_flag(&self, flag: &MovementFlags) -> bool {
        (self.movement_flags & (*flag as u32)) > 0 || (self.movement_flags2 & (*flag as u16)) > 0
    }

    pub fn has_any_movement_flag(&self, flags: &[MovementFlags]) -> bool {
        flags.iter().any(|f| self.has_movement_flag(f))
    }
}

pub trait ReadMovementInfo {
    fn read_movement_info(&mut self) -> Result<MovementInfo>;
}

impl<R: std::io::Read> ReadMovementInfo for R {
    fn read_movement_info(&mut self) -> Result<MovementInfo> {
        use podio::{LittleEndian, ReadPodExt};

        let info = MovementInfo {
            movement_flags: self.read_u32::<LittleEndian>()?,
            movement_flags2: self.read_u16::<LittleEndian>()?,
            time: self.read_u32::<LittleEndian>()?,
            position: self.read_position_and_orientation()?,
            falltime: self.read_u32::<LittleEndian>()?,
        };

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
