use crate::prelude::*;
use instance_manager::InstanceManager;
use std::sync::Arc;
use wrath_realm_db::RealmDatabase;

mod instance_manager;
//pub mod item_value_helpers;//Disabled because main::item is disabled
pub mod game_object;
mod map_manager;
mod update_builder;

pub mod prelude {
    pub use super::super::constants::*;
    pub use super::game_object::*;
    pub use super::map_manager::*;
    pub use super::update_builder::*;
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
