use crate::prelude::*;
use bit_field::BitArray;
use std::convert::TryInto;
use wrath_realm_db::character::DBCharacter;

pub struct TutorialFlags {
    pub flag_data: [u8; 32],
}

impl BitArray<u8> for TutorialFlags {
    fn bit_length(&self) -> usize {
        self.flag_data.bit_length()
    }

    fn get_bit(&self, bit: usize) -> bool {
        self.flag_data.get_bit(bit)
    }

    fn get_bits<U: std::ops::RangeBounds<usize>>(&self, range: U) -> u8 {
        self.flag_data.get_bits(range)
    }

    fn set_bit(&mut self, bit: usize, value: bool) {
        self.flag_data.set_bit(bit, value)
    }

    fn set_bits<U: std::ops::RangeBounds<usize>>(&mut self, range: U, value: u8) {
        self.flag_data.set_bits(range, value)
    }
}

impl<T> Into<TutorialFlags> for [T; 32]
where
    T: Into<u8>,
{
    fn into(self) -> TutorialFlags {
        TutorialFlags {
            flag_data: self.map(|a| a.into()),
        }
    }
}

impl TutorialFlags {
    pub fn from_database_entry(database_character_info: &DBCharacter) -> Result<Self> {
        let tutdat = database_character_info.tutorial_data.as_slice();
        let flag_data: [u8; 32];
        if tutdat.len() != 32 {
            flag_data = [0u8; 32];
        } else {
            flag_data = tutdat.try_into()?;
        }

        Ok(Self { flag_data })
    }
}
