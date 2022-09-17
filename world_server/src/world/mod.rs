use crate::prelude::*;
use instance_manager::InstanceManager;
use std::sync::Arc;
use wrath_realm_db::RealmDatabase;

//pub mod character_value_helpers;
mod instance_manager;
//pub mod item_value_helpers;//Disabled because main::item is disabled
pub mod map_cell;
mod map_manager;
pub mod map_object;
//pub mod unit_value_helpers;
//pub mod update_builder;
pub mod value_fields;

pub mod prelude {
    pub use super::super::constants::updates::*;
    pub use super::super::constants::*;
    //pub use super::character_value_helpers::CharacterValueHelpers;
    pub use super::map_cell::*;
    pub use super::map_manager::*;
    pub use super::map_object::*;
    //pub use super::unit_value_helpers::UnitValueHelpers;
    //pub use super::update_builder::*;
    pub use super::value_fields::*;
    pub use super::World;
}

pub struct World {
    instance_manager: Arc<InstanceManager>,
    realm_db: Arc<RealmDatabase>,
}

impl World {
    pub fn new(realm_db: Arc<RealmDatabase>) -> Self {
        Self {
            instance_manager: Arc::new(InstanceManager::new()),
            realm_db,
        }
    }

    pub fn get_instance_manager(&self) -> Arc<InstanceManager> {
        self.instance_manager.clone()
    }

    pub fn get_realm_database(&self) -> Arc<RealmDatabase> {
        self.realm_db.clone()
    }

    pub async fn tick(&self, delta_time: f32) -> Result<()> {
        self.instance_manager.tick(delta_time).await?;
        Ok(())
    }
}
