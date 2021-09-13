use std::sync::Arc;

use anyhow::Result;

use instance_manager::InstanceManager;

mod instance_manager;
pub mod map_cell;
mod map_manager;
pub mod map_object;
mod update_builder;
mod value_fields;

pub mod prelude {
    pub use super::super::constants::*;
    pub use super::map_cell::*;
    pub use super::map_object::*;
    pub use super::update_builder::*;
    pub use super::value_fields::*;
}

pub struct World {
    instance_manager: Arc<InstanceManager>,
}

impl World {
    pub fn new() -> Self {
        Self {
            instance_manager: Arc::new(InstanceManager::new()),
        }
    }

    pub fn get_instance_manager(&self) -> Arc<InstanceManager> {
        self.instance_manager.clone()
    }

    pub async fn tick(&self, _delta_time: f32) -> Result<()> {
        Ok(())
    }
}
