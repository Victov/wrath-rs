use chrono::prelude::*;
pub struct PackedTime(u32);

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

impl From<PackedTime> for u32 {
    fn from(p: PackedTime) -> Self {
        p.0
    }
}
