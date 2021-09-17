use std::sync::{Arc, Weak};
use std::time::Duration;

use super::map_object::MapObject;
use super::prelude::HasValueFields;
use super::update_builder::*;
use crate::prelude::*;
use async_std::sync::RwLock;
use kdtree::KdTree;

const NUM_MAP_DIMENSIONS: usize = 2;
const PUSH_WRITE_LOCK_TIMEOUT: Duration = Duration::from_secs(3);

pub struct MapManager {
    kd_tree: RwLock<KdTree<f32, Weak<RwLock<dyn MapObject>>, [f32; NUM_MAP_DIMENSIONS]>>,
}

impl MapManager {
    pub fn new() -> Self {
        Self {
            kd_tree: RwLock::new(KdTree::new(NUM_MAP_DIMENSIONS)),
        }
    }

    pub async fn push_object<T>(&self, object_ref: Weak<RwLock<T>>) -> Result<()>
    where
        T: MapObject + ReceiveUpdates + HasValueFields + 'static,
    {
        if let Some(object) = object_ref.upgrade() {
            let mut object = async_std::future::timeout(PUSH_WRITE_LOCK_TIMEOUT, object.write())
                .await
                .or_else(|_| bail!("Failed to get a write lock on an object to push to the map within reasonable time"))?;

            self.kd_tree
                .write()
                .await
                .add([object.get_position().x, object.get_position().y], object_ref.clone())?;

            if object.is_player() {
                let (block_count, mut update_data) = build_create_update_block_for_player(&*object, &*object)?;
                object.push_creation_data(&mut update_data, block_count);
            }
        } else {
            bail!("Recieved a weak ref to a character that no longer exists");
        }

        Ok(())
    }

    async fn _update_in_range_set<O, P>(&self, _object: Arc<RwLock<O>>, _player: Arc<RwLock<P>>) -> Result<()>
    where
        O: MapObject,
        P: MapObject + ReceiveUpdates,
    {
        Ok(())
    }
}
