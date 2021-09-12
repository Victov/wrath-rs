use anyhow::Result;
use async_std::sync::RwLock;

use crate::character::Character;
use crate::handlers;

use super::map_cell::*;
use super::map_object::MapObject;
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

    pub async fn push_object(&self, object: &mut (impl MapObject + ReceiveUpdates)) -> Result<()> {
        self.cell.write().await.push_object(object).await?;
        let (block_count, mut update_data) = build_create_update_block_for_player(object, object)?;
        object.push_creation_data(&mut update_data, block_count);

        /*
        let (num, buf) = object.get_creation_data();

        println!("creation data of {} blocks", num);
        for i in buf {
            println!("0x{:02x}", i);
        }*/

        Ok(())
    }
}
