use crate::prelude::*;
use instance_manager::InstanceManager;
use std::sync::Arc;

pub mod character_value_helpers;
mod instance_manager;
pub mod map_cell;
mod map_manager;
pub mod map_object;
pub mod unit_value_helpers;
pub mod update_builder;
pub mod value_fields;

pub mod prelude {
    pub use super::super::constants::updates::*;
    pub use super::super::constants::*;
    pub use super::character_value_helpers::CharacterValueHelpers;
    pub use super::map_cell::*;
    pub use super::map_manager::*;
    pub use super::map_object::*;
    pub use super::unit_value_helpers::UnitValueHelpers;
    pub use super::update_builder::*;
    pub use super::value_fields::*;
    pub use super::World;
}

#[derive(Default)]
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

    pub async fn tick(&self, delta_time: f32) -> Result<()> {
        self.instance_manager.tick(delta_time).await?;
        Ok(())
    }
}
