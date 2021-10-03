use super::value_fields::{HasValueFields, UnitFields};
use crate::{character::Character, prelude::*};

pub trait CharacterValueFields: HasValueFields {
    fn set_race(&mut self, race: u8) -> Result<()> {
        self.set_byte(UnitFields::UnitBytes0 as usize, 0, race)
    }

    fn set_class(&mut self, class: u8) -> Result<()> {
        self.set_byte(UnitFields::UnitBytes0 as usize, 1, class)
    }

    fn set_gender(&mut self, gender: u8) -> Result<()> {
        self.set_byte(UnitFields::UnitBytes0 as usize, 2, gender)
    }

    fn set_power_type(&mut self, power_type: u8) -> Result<()> {
        self.set_byte(UnitFields::UnitBytes0 as usize, 3, power_type)
    }
}

impl CharacterValueFields for Character {}
