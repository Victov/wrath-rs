use super::value_fields::{HasValueFields, UnitFields, UnitFlags};
use crate::constants::stand_state::UnitStandState;
use crate::{character::Character, prelude::*};

pub trait UnitValueHelpers: HasValueFields {
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

    fn set_stunned(&mut self, stunned: bool) -> Result<()> {
        self.set_unit_flag(UnitFlags::Stunned, stunned)
    }

    fn set_stand_state(&mut self, state: UnitStandState) -> Result<()> {
        self.set_byte(UnitFields::UnitBytes1 as usize, 0, state as u8)
    }
}

impl UnitValueHelpers for Character {}
