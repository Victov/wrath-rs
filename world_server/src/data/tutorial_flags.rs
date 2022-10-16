use crate::prelude::*;
use bit_field::BitArray;
use std::convert::{TryFrom, TryInto};
use wrath_realm_db::character::DBCharacter;

#[derive(Default)]
pub struct TutorialFlags {
    pub flag_data: [u32; 8],
}

impl BitArray<u32> for TutorialFlags {
    fn bit_length(&self) -> usize {
        self.flag_data.bit_length()
    }

    fn get_bit(&self, bit: usize) -> bool {
        self.flag_data.get_bit(bit)
    }

    fn get_bits<U: std::ops::RangeBounds<usize>>(&self, range: U) -> u32 {
        self.flag_data.get_bits(range)
    }

    fn set_bit(&mut self, bit: usize, value: bool) {
        self.flag_data.set_bit(bit, value)
    }

    fn set_bits<U: std::ops::RangeBounds<usize>>(&mut self, range: U, value: u32) {
        self.flag_data.set_bits(range, value)
    }
}

impl<T> From<&[T; 8]> for TutorialFlags
where
    T: Into<u32> + Copy,
{
    fn from(data: &[T; 8]) -> Self {
        TutorialFlags {
            flag_data: data.map(|a| a.into()),
        }
    }
}

impl<T> From<&[T; 32]> for TutorialFlags
where
    T: Into<u8> + Copy,
{
    fn from(data: &[T; 32]) -> Self {
        let mut temp_flag_data = [0u32; 8];
        for (index, int_val) in temp_flag_data.iter_mut().enumerate() {
            *int_val = u32::from_le_bytes([
                data[index * 4].into(),
                data[index * 4 + 1].into(),
                data[index * 4 + 2].into(),
                data[index * 4 + 3].into(),
            ]);
        }

        Self { flag_data: temp_flag_data }
    }
}

impl<T> TryFrom<&[T]> for TutorialFlags
where
    T: Into<u8> + Copy,
{
    type Error = anyhow::Error;
    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        let to_vec: Vec<u8> = value.iter().map(|a| (*a).into()).collect();
        let to_fixed_size: &[u8; 32] = &to_vec.try_into().map_err(|_| anyhow!("Wrong size"))?;
        Ok(Self::from(to_fixed_size))
    }
}

impl TutorialFlags {
    pub fn from_database_entry(database_character_info: &DBCharacter) -> Result<Self> {
        let db_tutorial_data: &[u8] = database_character_info.tutorial_data.as_slice();

        if db_tutorial_data.len() != 32 {
            bail!("Incorrect database array size")
        } else {
            let res: TutorialFlags = db_tutorial_data.try_into()?;
            Ok(res)
        }
    }

    pub fn reset(&mut self) {
        self.flag_data = [0; 8];
    }
}
