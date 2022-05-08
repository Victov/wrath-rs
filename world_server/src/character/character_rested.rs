use crate::prelude::*;
use crate::world::prelude::*;

#[derive(PartialEq, Debug)]
pub(super) enum RestedState {
    NotRested,
    Rested(RestedLocation),
}

#[derive(PartialEq, Debug)]
#[allow(dead_code)]
pub enum RestedLocation {
    City,
    Inn,
}

impl super::Character {
    pub fn handle_enter_inn(&mut self) -> Result<()> {
        if self.rested_state == RestedState::NotRested {
            self.rested_state = RestedState::Rested(RestedLocation::Inn);
            self.set_rested_bytes(true)?;
        }
        Ok(())
    }

    pub fn is_in_rested_area(&self) -> bool {
        match &self.rested_state {
            RestedState::NotRested => false,
            RestedState::Rested(location) => match location {
                RestedLocation::Inn => true,
                RestedLocation::City => true,
                //TODO still rested but outdoors => false
            },
        }
    }
}
