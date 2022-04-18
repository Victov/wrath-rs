use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use podio::{LittleEndian, ReadPodExt};
use std::collections::HashMap;
use std::fmt::Debug;

pub mod chr_races;
pub use chr_races::*;

pub mod chr_classes;
pub use chr_classes::*;

//See: https://wowdev.wiki/DBC
#[derive(Debug, Default)]
pub struct DBCHeader {
    magic: u32,
    rows_count: u32,
    columns_count: u32,
    row_size: u32,
    string_block_size: u32,
}

pub trait DBCTable {
    type RowType: DBCRowType;

    fn get_dbc_filename() -> &'static str
    where
        Self: Sized;
}

pub trait DBCRowType: Debug {
    type PrimaryKeyType: Eq + std::hash::Hash;

    fn read_row<T: std::io::Read>(reader: &mut T) -> Result<Self>
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
}

use chr_races::DBCCharRaces;
impl DBCStorage {
    pub fn new(dbc_files_path: String) -> Self {
        DBCStorage {
            dbc_files_path,
            chr_races: None,
            chr_classes: None,
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
        let mut header = DBCHeader::default();
        header.magic = reader.read_u32::<LittleEndian>()?;
        header.rows_count = reader.read_u32::<LittleEndian>()?;
        header.columns_count = reader.read_u32::<LittleEndian>()?;
        header.row_size = reader.read_u32::<LittleEndian>()?;
        header.string_block_size = reader.read_u32::<LittleEndian>()?;

        let mut rows = HashMap::<<T::RowType as DBCRowType>::PrimaryKeyType, T::RowType>::new();
        for _ in 0..header.rows_count {
            let row = <T::RowType as DBCRowType>::read_row(&mut reader)?;
            rows.insert(row.get_primary_key(), row);
        }

        Ok(DBCFile { header, rows })
    }
}
