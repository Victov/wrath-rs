use crate::prelude::*;

#[derive(PartialEq, Debug)]
pub(super) enum RestedState {
    NotRested,
    Rested(RestedLocation),
}

#[derive(PartialEq, Debug)]
#[allow(dead_code)]
pub enum RestedLocation {
    City, //Unused until server-side maps are implemented and the server can know when a character enters a city.
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
        /* Right now, after entering an inn once, the character will stay rested for the rest of
         * the session. The server needs to check area changes on tick, for which server-side maps
         * need to be implemented. Until that is done, un-resting will not work */

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
