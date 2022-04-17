use anyhow::anyhow;
use anyhow::Result;
use async_std::{fs::File, io::ReadExt};
use podio::{LittleEndian, ReadPodExt};
use std::collections::HashMap;
use std::fmt::Debug;

pub mod chr_races;
pub use chr_races::DBCCharRacesRow;

//See: https://wowdev.wiki/DBC
#[derive(Debug, Default)]
pub struct DBCHeader {
    magic: u32,
    rows_count: u32,
    columns_count: u32,
    row_size: u32,
    string_block_size: u32,
}

pub trait DBCRowType<'a>: Debug {
    fn get_dbc_filename() -> &'a str
    where
        Self: Sized;

    fn read_row<T: std::io::Read>(reader: &mut T) -> Result<Box<Self>>
    where
        Self: Sized;
}

pub struct DBCFile<'a> {
    pub header: DBCHeader,
    rows: Vec<Box<dyn DBCRowType<'a>>>,
}

type DBCFilesMap<'a> = HashMap<&'a str, DBCFile<'a>>;
pub struct DBCStorage<'a> {
    dbc_files_path: &'a str,
    files: DBCFilesMap<'a>,
}

impl<'a> DBCStorage<'a> {
    pub fn new(dbc_files_path: &'a str) -> Self {
        DBCStorage {
            dbc_files_path,
            files: DBCFilesMap::default(),
        }
    }

    pub async fn get_dbc<T: DBCRowType<'a> + Debug + 'static>(&mut self) -> Result<&DBCFile<'a>> {
        if self.files.contains_key(T::get_dbc_filename()) {
            return self
                .files
                .get(T::get_dbc_filename())
                .ok_or(anyhow!("Failed to get DBC"));
        } else {
            let dbc = self.load_dbc::<T>().await?;
            self.files.insert(T::get_dbc_filename(), dbc);
            return self
                .files
                .get(T::get_dbc_filename())
                .ok_or(anyhow!("failed to get DBC"));
        }
    }

    async fn load_dbc<T: DBCRowType<'a> + Debug + 'static>(&mut self) -> Result<DBCFile<'a>> {
        use async_std::io::BufReader;
        use async_std::path::PathBuf;

        let filename = T::get_dbc_filename();
        let mut buffer = Vec::<u8>::new();
        {
            let path: PathBuf = [self.dbc_files_path, filename].iter().collect();
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

        let mut rows = vec![];
        for _ in 0..header.rows_count {
            let row: Box<dyn DBCRowType<'a> + 'static> = T::read_row(&mut reader)?;
            println!("Row: {:?}", row);
            rows.push(row);
        }

        Ok(DBCFile { header, rows })
    }
}
