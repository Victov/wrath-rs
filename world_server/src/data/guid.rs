use podio::ReadPodExt;

use crate::prelude::*;
use std::fmt::Display;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Guid(u64);

//from: https://github.com/azerothcore/azerothcore-wotlk/blob/master/src/server/game/Entities/Object/ObjectDefines.h
#[allow(dead_code)]
pub enum HighGuid {
    Player = 0x0000,          // blizz 0000
    ItemOrContainer = 0x4000, // blizz 4000
    GameObject = 0xF110,      // blizz F110
    Transport = 0xF120,       // blizz F120 (for GAMEOBJECT_TYPE_TRANSPORT)
    Unit = 0xF130,            // blizz F130
    Pet = 0xF140,             // blizz F140
    Vehicle = 0xF150,         // blizz F550
    DynamicObject = 0xF100,   // blizz F100
    Corpse = 0xF101,          // blizz F100
    MoTransport = 0x1FC0,     // blizz 1FC0 (for GAMEOBJECT_TYPE_MO_TRANSPORT)
    Group = 0x1F50,
    Instance = 0x1F42, // blizz 1F42/1F44/1F44/1F47
}

impl Guid {
    pub fn new(low: u32, high: HighGuid) -> Self {
        Self((low as u64) | (high as u64) << 32)
    }

    pub fn get_low_part(&self) -> u32 {
        self.0 as u32
    }

    pub fn get_high_part(&self) -> u32 {
        ((self.0 & 0xFFFFFFFF00000000) >> 32) as u32
    }
}

pub trait WriteGuid {
    fn write_guid<T: podio::Endianness>(&mut self, guid: &Guid) -> Result<()>;
    fn write_guid_compressed(&mut self, guid: &Guid) -> Result<()>;
}

impl<W: std::io::Write> WriteGuid for W {
    fn write_guid<T: podio::Endianness>(&mut self, guid: &Guid) -> Result<()> {
        use podio::WritePodExt;
        self.write_u64::<T>(guid.0)?;
        Ok(())
    }

    fn write_guid_compressed(&mut self, guid: &Guid) -> Result<()> {
        let mut mask: u8 = 0;
        let inner: u64 = guid.0;
        for i in 0..8 {
            if get_byte_value_at(inner, i) > 0 {
                mask |= 1 << i;
            }
        }

        use podio::WritePodExt;
        self.write_u8(mask)?;
        for i in 0..8 {
            let val = get_byte_value_at(inner, i);
            if val > 0 {
                self.write_u8(val)?;
            }
        }

        Ok(())
    }
}

fn get_byte_value_at(input: u64, index: isize) -> u8 {
    let shifted = input >> (8 * index);
    let result = (shifted & 0x00000000000000FF) as u8;

    result
}

pub trait ReadGuid {
    fn read_guid<T: podio::Endianness>(&mut self) -> Result<Guid>;
    fn read_guid_compressed(&mut self) -> Result<Guid>;
}

impl<R: std::io::Read> ReadGuid for R {
    fn read_guid<T: podio::Endianness>(&mut self) -> Result<Guid> {
        let val = self.read_u64::<T>()?;
        Ok(Guid(val))
    }

    fn read_guid_compressed(&mut self) -> Result<Guid> {
        use bit_field::BitField;
        let mask = self.read_u8()?;
        let mut guid = 0u64;
        for i in 0..8 {
            if mask.get_bit(i) {
                let byte = self.read_u8()?;
                guid |= (byte as u64) << (i * 8);
            }
        }
        Ok(guid.into())
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Guid(0x{:016x})", self.0)
    }
}

impl From<u64> for Guid {
    fn from(val: u64) -> Self {
        Self(val)
    }
}