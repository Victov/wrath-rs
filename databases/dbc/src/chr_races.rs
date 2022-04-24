use crate::StringTable;

use super::ReadSkip;
use anyhow::Result;

#[derive(Debug)]
pub struct DBCCharRaces;

#[derive(Debug)]
pub struct DBCCharRacesRow {
    pub race_id: u32,
    pub male_model_id: u32,
    pub female_model_id: u32,
    pub required_expansion: u32,
}

impl super::DBCTable for DBCCharRaces {
    type RowType = DBCCharRacesRow;

    fn get_dbc_filename() -> &'static str
    where
        Self: Sized,
    {
        &"ChrRaces.dbc"
    }
}

impl super::DBCRowType for DBCCharRacesRow {
    type PrimaryKeyType = u32;

    fn read_row<T: std::io::Read>(reader: &mut T, _string_table: &StringTable) -> Result<Self>
    where
        Self: Sized,
    {
        use podio::{LittleEndian, ReadPodExt};

        let race_id = reader.read_u32::<LittleEndian>()?;
        let _flags = reader.read_u32::<LittleEndian>()?;
        let _faction_id = reader.read_u32::<LittleEndian>()?;
        let _exploration_sound_id = reader.read_u32::<LittleEndian>()?;
        let male_model_id = reader.read_u32::<LittleEndian>()?;
        let female_model_id = reader.read_u32::<LittleEndian>()?;

        reader.skip::<u32>(62)?; //skip 62 u32 fields of no interest
        let required_expansion = reader.read_u32::<LittleEndian>()?;

        Ok(DBCCharRacesRow {
            race_id,
            male_model_id,
            female_model_id,
            required_expansion,
        })
    }

    fn get_primary_key(&self) -> Self::PrimaryKeyType {
        self.race_id
    }
}
