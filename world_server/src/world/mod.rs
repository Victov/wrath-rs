use crate::{character::character_manager::CharacterManager, prelude::*};
use instance_manager::InstanceManager;
use std::sync::Arc;
use wrath_realm_db::RealmDatabase;

pub mod game_object;
mod instance_manager;
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
    instance_manager: InstanceManager,
    realm_db: Arc<RealmDatabase>,
}

impl World {
    pub fn new(realm_db: Arc<RealmDatabase>) -> Self {
        Self {
            instance_manager: InstanceManager::new(),
            realm_db,
        }
    }

    pub fn get_instance_manager(&self) -> &InstanceManager {
        &self.instance_manager
    }

    pub fn get_instance_manager_mut(&mut self) -> &mut InstanceManager {
        &mut self.instance_manager
    }

    pub fn get_realm_database(&self) -> Arc<RealmDatabase> {
        self.realm_db.clone()
    }

    pub async fn tick(&mut self, character_manager: &mut CharacterManager, delta_time: f32) -> Result<()> {
        self.instance_manager.tick(character_manager, delta_time).await?;
        Ok(())
    }
}
