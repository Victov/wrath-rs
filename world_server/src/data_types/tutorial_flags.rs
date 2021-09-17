use crate::prelude::*;
use std::convert::TryInto;
use wrath_realm_db::character::DBCharacter;

pub struct TutorialFlags {
    pub flag_data: [u8; 32],
}

impl<T> Into<TutorialFlags> for [T; 32]
where
    T: Into<u8>,
{
    fn into(self) -> TutorialFlags {
        TutorialFlags {
            flag_data: self.map(|a| a.into()).into(),
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
