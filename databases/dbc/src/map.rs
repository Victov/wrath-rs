use crate::StringTable;

use super::ReadSkip;
use anyhow::Result;

#[derive(Debug)]
pub struct DBCMap;

#[derive(Debug)]
pub struct DBCMapRow {
    pub map_id: u32,
    pub map_type: u32,
    pub name: String,
}

impl DBCMapRow {
    pub fn is_dungeon(&self) -> bool {
        self.map_type == 1
    }

    pub fn is_raid(&self) -> bool {
        self.map_type == 2
    }

    pub fn is_battleground(&self) -> bool {
        self.map_type == 3
    }

    pub fn is_arena(&self) -> bool {
        self.map_type == 4
    }

    pub fn is_dungeon_or_raid(&self) -> bool {
        self.is_dungeon() || self.is_raid()
    }
}

impl super::DBCTable for DBCMap {
    type RowType = DBCMapRow;

    fn get_dbc_filename() -> &'static str
    where
        Self: Sized,
    {
        "Map.dbc"
    }
}

impl super::DBCRowType for DBCMapRow {
    type PrimaryKeyType = u32;

    fn read_row<T: std::io::Read>(reader: &mut T, string_table: &StringTable) -> Result<Self>
    where
        Self: Sized,
    {
        use podio::{LittleEndian, ReadPodExt};

        let map_id = reader.read_u32::<LittleEndian>()?;
        reader.skip::<u32>(1)?;
        let map_type = reader.read_u32::<LittleEndian>()?;
        assert!(map_type < 5);
        reader.skip::<u32>(2)?;

        let name_index = reader.read_u32::<LittleEndian>()?;
        let name = string_table
            .strings
            .get(&name_index)
            .unwrap_or(&String::from("invalid"))
            .clone();

        reader.skip::<u32>(60)?;

        Ok(DBCMapRow {
            map_id,
            map_type,
            name,
        })
    }

    fn get_primary_key(&self) -> Self::PrimaryKeyType {
        self.map_id
    }
}
