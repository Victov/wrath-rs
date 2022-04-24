use super::ReadSkip;
use anyhow::Result;

#[derive(Debug)]
pub struct DBCCharClasses;

#[derive(Debug)]
pub struct DBCCharClassesRow {
    pub class_id: u32,
    pub power_type: u32,
    pub required_expansion: u32,
}

impl super::DBCTable for DBCCharClasses {
    type RowType = DBCCharClassesRow;

    fn get_dbc_filename() -> &'static str
    where
        Self: Sized,
    {
        &"ChrClasses.dbc"
    }
}

impl super::DBCRowType for DBCCharClassesRow {
    type PrimaryKeyType = u32;

    fn read_row<T: std::io::Read>(reader: &mut T) -> Result<Self>
    where
        Self: Sized,
    {
        use podio::{LittleEndian, ReadPodExt};

        let class_id = reader.read_u32::<LittleEndian>()?;
        let _unk1 = reader.read_u32::<LittleEndian>()?;
        let power_type = reader.read_u32::<LittleEndian>()?;

        reader.skip::<u32>(56)?; //skip 56 u32 fields of no interest
        let required_expansion = reader.read_u32::<LittleEndian>()?;

        Ok(DBCCharClassesRow {
            class_id,
            power_type,
            required_expansion,
        })
    }

    fn get_primary_key(&self) -> Self::PrimaryKeyType {
        self.class_id
    }
}
