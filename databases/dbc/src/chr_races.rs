use anyhow::Result;

#[derive(Debug)]
pub struct DBCCharRacesRow {
    pub race_id: u32,
    pub male_model_id: u32,
    pub female_model_id: u32,
    pub required_expansion: u32,
}

impl<'a> super::DBCRowType<'a> for DBCCharRacesRow {
    fn get_dbc_filename() -> &'a str
    where
        Self: Sized,
    {
        &"ChrRaces.dbc"
    }

    fn read_row<T: std::io::Read>(reader: &mut T) -> Result<Box<Self>>
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

        //Have to tell the compiler to not use the default Read
        <T as ReadPodExt>::read_exact(reader, 62 * 4)?; //skip 62 u32 fields of no interest
        let required_expansion = reader.read_u32::<LittleEndian>()?;

        Ok(Box::new(DBCCharRacesRow {
            race_id,
            male_model_id,
            female_model_id,
            required_expansion,
        }))
    }
}
