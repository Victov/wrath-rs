use std::collections::HashMap;
use std::sync::Weak;

use super::map_object::MapObject;
use super::prelude::*;
use crate::prelude::*;
use async_std::sync::RwLock;
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;

const NUM_MAP_DIMENSIONS: usize = 2;

pub struct MapManager {
    objects_on_map: RwLock<HashMap<Guid, Weak<RwLock<dyn MapObjectWithValueFields>>>>,
    kd_tree: RwLock<KdTree<f32, Weak<RwLock<dyn MapObjectWithValueFields>>, [f32; NUM_MAP_DIMENSIONS]>>,
}

impl MapManager {
    pub fn new() -> Self {
        Self {
            objects_on_map: RwLock::new(HashMap::new()),
            kd_tree: RwLock::new(KdTree::new(NUM_MAP_DIMENSIONS)),
        }
    }

    pub async fn tick(&self, _delta_time: f32) -> Result<()> {
        let map_objects = self.objects_on_map.read().await;
        for map_object in (*map_objects).values() {
            if let Some(valid_object_lock) = map_object.upgrade() {
                let mut valid_object = valid_object_lock.write().await;
                if valid_object.wants_updates() {
                    valid_object.as_update_receiver_mut().unwrap().process_pending_updates().await?;
                }
            }
        }

        Ok(())
    }

    pub async fn push_object<T>(&self, object_ref: Weak<RwLock<T>>) -> Result<()>
    where
        T: MapObject + HasValueFields + 'static,
    {
        if let Some(object_lock) = object_ref.upgrade() {
            {
                let object = object_lock.read().await;
                self.kd_tree
                    .write()
                    .await
                    .add([object.get_position().x, object.get_position().y], object_ref.clone())?;

                self.objects_on_map.write().await.insert(object.get_guid().clone(), object_ref.clone());
            }

            object_lock.write().await.on_pushed_to_map(self)?;
            self.update_in_range_set(object_ref).await?;
        } else {
            bail!("Recieved a weak ref to a character that no longer exists");
        }

        Ok(())
    }

    async fn update_in_range_set(&self, object_ref: Weak<RwLock<impl MapObjectWithValueFields + 'static>>) -> Result<()> {
        let target_object_lock = object_ref.upgrade().unwrap();

        let tree = self.kd_tree.read().await;
        let within_range: Vec<(f32, &Weak<RwLock<dyn MapObjectWithValueFields>>)> = {
            let object = target_object_lock.read().await;
            let object_pos = &[object.get_position().x, object.get_position().y];
            tree.within(object_pos, 20.0f32, &squared_euclidean)?
        };

        for (range, lock_ptr) in within_range {
            if let Some(in_range_object_lock) = lock_ptr.upgrade() {
                //Only read locks required here
                {
                    let read_obj = in_range_object_lock.read().await;
                    let target_object = target_object_lock.read().await;
                    if read_obj.get_guid() == target_object.get_guid() {
                        //skip ourselves
                        continue;
                    }

                    if read_obj.is_in_range(target_object.get_guid()) {
                        //skip if we already know this object in our range
                        continue;
                    }
                    info!("New object in range! Dist: {}, Guid: {}", range, read_obj.get_guid());
                }

                {
                    let mut write_obj = in_range_object_lock.write().await;
                    write_obj.add_in_range_object(target_object_lock.read().await.get_guid(), object_ref.clone())?;
                    if write_obj.wants_updates() {
                        let (block_count, mut buffer) = build_create_update_block_for_player(&*write_obj, &*target_object_lock.read().await)?;
                        if let Some(wants_updates) = write_obj.as_update_receiver_mut() {
                            wants_updates.push_creation_data(&mut buffer, block_count);
                        }
                    }
                }
                {
                    let mut write_target_object = target_object_lock.write().await;
                    write_target_object.add_in_range_object(in_range_object_lock.read().await.get_guid(), lock_ptr.clone())?;
                    if write_target_object.wants_updates() {
                        let (block_count, mut buffer) =
                            build_create_update_block_for_player(&*write_target_object, &*in_range_object_lock.read().await)?;
                        if let Some(wants_updates) = write_target_object.as_update_receiver_mut() {
                            wants_updates.push_creation_data(&mut buffer, block_count);
                        }
                    }
                }
            } else {
                //The object should have been cleaned up, so at this point this is a warning
                //Don't do clean-up here. This exposes a problem elsewhere, don't try to fix it
                //here
                warn!("We got an object in range that's no longer active on the server, weakref failed to upgrade");
                continue;
            }
        }

        Ok(())
    }
}
