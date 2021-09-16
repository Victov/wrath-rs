use crate::prelude::*;
use chrono::prelude::*;
pub struct PackedTime(u32);

pub trait WritePackedTime {
    fn write_packed_time<T: podio::Endianness>(&mut self, packed_time: &PackedTime) -> Result<()>;
}

impl<W: std::io::Write> WritePackedTime for W {
    fn write_packed_time<T: podio::Endianness>(&mut self, packed_time: &PackedTime) -> Result<()> {
        use podio::WritePodExt;
        self.write_u32::<T>(packed_time.0)?;
        Ok(())
    }
}

pub trait ReadPackedTime {
    fn read_packed_time<T: podio::Endianness>(&mut self) -> Result<PackedTime>;
}

impl<R: std::io::Read> ReadPackedTime for R {
    fn read_packed_time<T: podio::Endianness>(&mut self) -> Result<PackedTime> {
        use podio::ReadPodExt;
        let val = self.read_u32::<T>()?;
        Ok(PackedTime(val))
    }
}

impl<T: TimeZone> From<DateTime<T>> for PackedTime {
    fn from(datetime: DateTime<T>) -> Self {
        let day_of_week = datetime.weekday() as u32; //TODO: from sunday? from monday? might need adjustment

        let mut game_time = datetime.minute() as u32 & 0x3F;
        game_time |= ((datetime.hour() as u32) << 6) & 0x7C0;
        game_time |= (day_of_week << 11) & 0x3800;
        game_time |= ((datetime.day0() as u32) << 14) & 0xFC000;
        game_time |= ((datetime.month0() as u32) << 20) & 0xF00000;
        game_time |= (((datetime.year() as u32) - 2000) << 24) & 0x1F000000;

        PackedTime(game_time)
    }
}
