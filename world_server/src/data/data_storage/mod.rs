use crate::prelude::*;
use async_std::{
    io::{BufReader, ReadExt},
    path::PathBuf,
};
use std::sync::Arc;
use wow_dbc::wrath_tables::{area_trigger::AreaTriggerKey, chr_classes::ChrClasses, chr_races::ChrRaces};
use wrath_realm_db::RealmDatabase;

mod area_triggers;
pub use area_triggers::*;

#[derive(Default)]
pub struct DataStorage {
    dbc_chr_races: Option<ChrRaces>,
    dbc_chr_classes: Option<ChrClasses>,
    dbc_chr_map: Option<wow_dbc::wrath_tables::map::Map>,
    dbc_char_start_outfit: Option<wow_dbc::wrath_tables::char_start_outfit::CharStartOutfit>,
    area_triggers: std::collections::hash_map::HashMap<AreaTriggerKey, AreaTrigger>,
}

async fn load_standard_dbc<T: wow_dbc::DbcTable>(folder_path: impl Into<&str>, table: &mut Option<T>) -> Result<()> {
    assert!(table.is_none());

    //Use async to read the file into memory
    let filename = T::filename();
    let path: PathBuf = [folder_path.into(), filename].iter().collect();
    info!("loading {}", path.to_str().unwrap());
    let file_handle = async_std::fs::File::open(path).await?;
    let mut reader = BufReader::new(file_handle);
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes).await?;

    //Since wow_dbc is not async-aware, use an std memory reader into our array to do the parsing
    let mut mem_reader = std::io::BufReader::new(bytes.as_slice());
    let res = T::read(&mut mem_reader);
    if let Ok(t) = res {
        *table = Some(t);
    } else if let Err(e) = res {
        error!("Failure while loading {}, {}", T::filename(), e);
    }
    Ok(())
}

macro_rules! define_dbc_getter {
    ($typename:path,$propname:ident,$fnname:ident) => {
        pub fn $fnname(&self) -> Result<&$typename> {
            self.$propname
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("DBC {} is not loaded yet", stringify!($typename)))
        }
    };
}

impl DataStorage {
    pub async fn load(&mut self, realm_db: Arc<RealmDatabase>) -> Result<()> {
        let dbc_path = &*std::env::var("DBC_FOLDER_PATH")?;
        info!("Loading DBC files from folder: {}", dbc_path);
        load_standard_dbc(dbc_path, &mut self.dbc_chr_races).await?;
        load_standard_dbc(dbc_path, &mut self.dbc_chr_classes).await?;
        load_standard_dbc(dbc_path, &mut self.dbc_chr_map).await?;
        load_standard_dbc(dbc_path, &mut self.dbc_char_start_outfit).await?;
        self.load_area_triggers(dbc_path, realm_db).await?;
        info!("Finished loading DBC files");
        info!("Loading SQL data");
        info!("Loading item templates");
        Ok(())
    }

    define_dbc_getter!(ChrRaces, dbc_chr_races, get_dbc_chr_races);
    define_dbc_getter!(ChrClasses, dbc_chr_classes, get_dbc_chr_classes);
    define_dbc_getter!(wow_dbc::wrath_tables::map::Map, dbc_chr_map, get_dbc_chr_map);
    define_dbc_getter!(
        wow_dbc::wrath_tables::char_start_outfit::CharStartOutfit,
        dbc_char_start_outfit,
        get_dbc_char_start_outfit
    );

    //Area triggers need special treatment from joint DBC and Mysql data sources, so they don't use
    //forward_dbc_getter
    pub fn get_area_trigger(&self, key: impl Into<AreaTriggerKey>) -> Option<&AreaTrigger> {
        self.area_triggers.get(&key.into())
    }
}
