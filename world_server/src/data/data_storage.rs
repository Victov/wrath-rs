use crate::prelude::*;
use dbc::DBCStorage;

#[derive(Default)]
pub struct DataStorage {
    dbc_storage: Option<DBCStorage>,
}

macro_rules! forward_dbc_getter {
    ($typename:path, $dbc_function_name:ident, $fnname: ident) => {
        pub fn $fnname(&self) -> &dbc::DBCFile<$typename> {
            self.dbc_storage.as_ref().unwrap().$dbc_function_name().unwrap()
        }
    };
}

impl DataStorage {
    pub async fn load(&mut self) -> Result<()> {
        let dbc_path = std::env::var("DBC_FOLDER_PATH")?;
        info!("Loading DBC files from folder: {}", dbc_path);
        let mut dbc_storage = DBCStorage::new(dbc_path);
        dbc_storage.load_dbc_char_races().await?;
        dbc_storage.load_dbc_char_classes().await?;
        dbc_storage.load_dbc_maps().await?;
        dbc_storage.load_dbc_area_triggers().await?;

        info!("Finished loading DBC files");
        self.dbc_storage = Some(dbc_storage);

        Ok(())
    }

    //If the DBC data needs to special treatment, we forward it right away
    //This will be the case for most of the DBCs
    forward_dbc_getter!(dbc::DBCCharRaces, get_dbc_char_races, get_char_races);
    forward_dbc_getter!(dbc::DBCCharClasses, get_dbc_char_classes, get_char_classes);
    forward_dbc_getter!(dbc::DBCMap, get_dbc_maps, get_maps);

    //Some DBC data needs to be appended with some database data or processed in some other way.
    //This facade module allows us to give it some special treatment.
}
