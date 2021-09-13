use anyhow::Result;
use std::convert::TryInto;
use wrath_realm_db::character::DBCharacter;

pub struct TutorialFlags {
    pub flag_data: [u8; 32],
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
