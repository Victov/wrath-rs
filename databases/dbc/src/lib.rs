use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use podio::{LittleEndian, ReadPodExt};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::BufRead;

mod chr_races;
pub use chr_races::*;

mod chr_classes;
pub use chr_classes::*;

mod map;
pub use map::*;

mod area_trigger;
pub use area_trigger::*;

pub(crate) mod helpers;
pub(crate) use helpers::ReadSkip;

//See: https://wowdev.wiki/DBC
#[derive(Debug, Default)]
pub struct DBCHeader {
    pub magic: u32,
    pub rows_count: u32,
    pub columns_count: u32,
    pub row_size: u32,
    pub string_block_size: u32,
}

pub trait DBCTable {
    type RowType: DBCRowType;

    fn get_dbc_filename() -> &'static str
    where
        Self: Sized;
}

pub struct StringTable {
    strings: HashMap<u32, String>,
}

pub trait DBCRowType: Debug {
    type PrimaryKeyType: Eq + std::hash::Hash;

    fn read_row<T: std::io::Read>(reader: &mut T, string_table: &StringTable) -> Result<Self>
    where
        Self: Sized;

    fn get_primary_key(&self) -> Self::PrimaryKeyType;
}

pub struct DBCFile<T: DBCTable> {
    pub header: DBCHeader,
    rows: HashMap<<T::RowType as DBCRowType>::PrimaryKeyType, T::RowType>,
}

impl<T: DBCTable> DBCFile<T> {
    pub fn get_entry(
        &self,
        key: <T::RowType as DBCRowType>::PrimaryKeyType,
    ) -> Option<&T::RowType> {
        self.rows.get(&key)
    }
}

macro_rules! define_dbc_getter {
    ($typename:path,$propname:ident,$fnname:ident) => {
        pub fn $fnname(&self) -> Result<&DBCFile<$typename>> {
            self.$propname
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("DBC $a is not loaded yet"))
        }
    };
}

macro_rules! define_dbc_loader {
    ($typename:path,$propname:ident,$fnname:ident) => {
        pub async fn $fnname(&mut self) -> Result<()> {
            let name = stringify!($fnname);
            assert!(
                self.$propname.is_none(),
                "DBC {} is already loaded",
                name.to_string()
            );
            self.$propname = Some(self.load_dbc::<$typename>().await?);
            Ok(())
        }
    };
}

macro_rules! define_dbc {
    ($typename:path, $propname:ident, $getter:ident, $loader:ident) => {
        define_dbc_getter!($typename, $propname, $getter);
        define_dbc_loader!($typename, $propname, $loader);
    };
}

pub struct DBCStorage {
    dbc_files_path: String,
    chr_races: Option<DBCFile<DBCCharRaces>>,
    chr_classes: Option<DBCFile<DBCCharClasses>>,
    maps: Option<DBCFile<DBCMap>>,
    area_triggers: Option<DBCFile<DBCAreaTrigger>>,
}

impl DBCStorage {
    pub fn new(dbc_files_path: String) -> Self {
        DBCStorage {
            dbc_files_path,
            chr_races: None,
            chr_classes: None,
            maps: None,
            area_triggers: None,
        }
    }

    define_dbc!(
        chr_races::DBCCharRaces,
        chr_races,
        get_dbc_char_races,
        load_dbc_char_races
    );

    define_dbc!(
        chr_classes::DBCCharClasses,
        chr_classes,
        get_dbc_char_classes,
        load_dbc_char_classes
    );

    define_dbc!(
        area_trigger::DBCAreaTrigger,
        area_triggers,
        get_dbc_area_triggers,
        load_dbc_area_triggers
    );

    define_dbc!(map::DBCMap, maps, get_dbc_maps, load_dbc_maps);

    async fn load_dbc<T: DBCTable + Debug>(&self) -> Result<DBCFile<T>> {
        use async_std::io::BufReader;
        use async_std::path::PathBuf;

        let filename = T::get_dbc_filename();
        let mut buffer = Vec::<u8>::new();
        {
            let path: PathBuf = [&self.dbc_files_path, filename].iter().collect();
            let file_handle = File::open(path).await?;
            let mut buf_reader = BufReader::new(file_handle);
            buf_reader.read_to_end(&mut buffer).await?;
        }

        let mut reader = std::io::Cursor::new(buffer);
        let header = DBCHeader {
            magic: reader.read_u32::<LittleEndian>()?,
            rows_count: reader.read_u32::<LittleEndian>()?,
            columns_count: reader.read_u32::<LittleEndian>()?,
            row_size: reader.read_u32::<LittleEndian>()?,
            string_block_size: reader.read_u32::<LittleEndian>()?,
        };

        //Skip ahead to start of string table
        reader.set_position(
            20u64 /*header size*/ + header.row_size as u64 * header.rows_count as u64,
        );

        //TODO: improve: since the string table is only used for reach row to do lookups
        //we may be able to keep the strings on the stack during parsing. (&str)
        //Each row can then turn it into a String _if_ they need the string
        //Since some rows don't even care about the strings, but we do always parse the string
        //table

        let mut strings = HashMap::default();
        let mut buf = vec![];
        let mut curr_offset = 0u32;
        loop {
            let read_bytes = reader.read_until(0, &mut buf)? as u32;
            if read_bytes == 0 {
                break;
            }
            curr_offset += read_bytes;
            let string = String::from_utf8(buf.clone())?
                .trim_matches(char::from(0))
                .into();
            strings.insert(curr_offset, string);
            buf.clear();
        }
        let string_table = StringTable { strings };

        //back to actual rows
        reader.set_position(20);

        let mut rows = HashMap::<<T::RowType as DBCRowType>::PrimaryKeyType, T::RowType>::new();
        for _ in 0..header.rows_count {
            let row = <T::RowType as DBCRowType>::read_row(&mut reader, &string_table)?;
            rows.insert(row.get_primary_key(), row);
        }

        Ok(DBCFile { header, rows })
    }
}
