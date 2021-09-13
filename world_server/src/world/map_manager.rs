use anyhow::Result;
use async_std::sync::RwLock;

use super::map_cell::*;
use super::map_object::MapObject;
use super::prelude::HasValueFields;
use super::update_builder::*;

pub struct MapManager {
    //Of course, the mapmananger should have multiple cells
    //but that's something for later. For now we just have our
    //one cozy cell and we disregard position and always return
    //this one cell
    cell: RwLock<MapCell>,
}

impl MapManager {
    pub fn new() -> Self {
        Self {
            cell: RwLock::new(MapCell::new()),
        }
    }

    pub async fn push_object(&self, object: &mut (impl MapObject + ReceiveUpdates + HasValueFields)) -> Result<()> {
        self.cell.write().await.push_object(object).await?;
        let (block_count, mut update_data) = build_create_update_block_for_player(object, object)?;
        object.push_creation_data(&mut update_data, block_count);

        Ok(())
    }
}
