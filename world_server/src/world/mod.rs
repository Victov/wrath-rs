use anyhow::Result;

mod map_manager;
use map_manager::MapManager;

pub struct World {
    pub map_manager: MapManager,
}

impl World {
    pub fn new() -> Self {
        Self {
            map_manager: MapManager::new(),
        }
    }

    pub async fn tick(&self, _delta_time: f32) -> Result<()> {
        Ok(())
    }
}
