use super::instance_manager::MapID;
use std::collections::HashMap;
use std::sync::Weak;

use super::prelude::GameObject;
use super::prelude::*;
use crate::prelude::*;
use async_std::sync::{Mutex, RwLock};
use rstar::{PointDistance, RTree, RTreeObject, AABB};

const VISIBILITY_RANGE: f32 = 5000.0f32;

#[derive(Clone, Copy, PartialEq, Debug)]
struct RStarTreeItem {
    x: f32,
    y: f32,
    guid: Guid,
}

impl RTreeObject for RStarTreeItem {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for RStarTreeItem {
    fn distance_2(&self, point: &<Self::Envelope as rstar::Envelope>::Point) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
        self.envelope().distance_2(point)
    }
}
pub struct MapManager {
    id: MapID,
    objects_on_map: RwLock<HashMap<Guid, Weak<RwLock<dyn GameObject>>>>,
    query_tree: RwLock<RTree<RStarTreeItem>>,
    add_queue: Mutex<Vec<Weak<RwLock<dyn GameObject>>>>,
    remove_queue: Mutex<Vec<Guid>>,
}

impl MapManager {
    pub fn new(id: MapID) -> Self {
        info!("spawned new map with id {}", id);
        Self {
            id,
            objects_on_map: RwLock::new(HashMap::new()),
            query_tree: RwLock::new(RTree::new()),
            add_queue: Mutex::new(Vec::new()),
            remove_queue: Mutex::new(Vec::new()),
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Map {} shutting down", self.id);
        Ok(())
    }

    pub async fn should_shutdown(&self) -> bool {
        self.objects_on_map.read().await.len() == 0 && self.add_queue.lock().await.len() == 0
    }

    pub async fn try_get_object(&self, guid: &Guid) -> Option<Weak<RwLock<dyn GameObject>>> {
        let map_objects = self.objects_on_map.read().await;
        map_objects.get(guid).cloned()
    }

    pub async fn tick(&self, _delta_time: f32) -> Result<()> {
        self.rebuild_object_querying_tree().await?;

        let any_removed = self.process_remove_queue().await?;
        let any_added = self.process_add_queue().await?;

        let map_objects = self.objects_on_map.read().await;
        for map_object in (*map_objects).values() {
            self.update_in_range_set(map_object.clone()).await?;

            if let Some(valid_object_lock) = map_object.upgrade() {
                let mut valid_object = valid_object_lock.write().await;
                if valid_object.as_world_object().map(|a| a.wants_updates()).unwrap_or(false) {
                    let has_any_update_bit = valid_object
                        .as_has_value_fields()
                        .map(|a| a.get_update_mask().has_any_bit())
                        .unwrap_or(false);

                    let has_something_recently_removed = valid_object
                        .as_world_object()
                        .map(|a| !a.get_recently_removed_range_guids().is_empty())
                        .unwrap_or(false);

                    if has_any_update_bit || any_removed || any_added || has_something_recently_removed {
                        //Hard unwrap should be safe here
                        let (num_blocks, mut buf) = build_out_of_range_update_block_for_player(&*valid_object.as_world_object().unwrap())?;
                        valid_object.as_world_object_mut().unwrap().clear_recently_removed_range_guids()?;
                        valid_object.as_update_receiver_mut().unwrap().push_update_block(&mut buf, num_blocks);

                        let (num_blocks, buf) =
                            build_values_update_block(valid_object.as_map_object().get_guid(), &*valid_object.as_has_value_fields().unwrap())?;

                        if let Some(valid_object) = valid_object.as_update_receiver_mut() {
                            valid_object.push_update_block(&mut buf.clone(), num_blocks);
                        }

                        let in_range_guids = valid_object.as_world_object().unwrap().get_in_range_guids();
                        for in_range_guid in in_range_guids {
                            if let Some(in_range_object_weak) = map_objects.get(in_range_guid) {
                                if let Some(in_range_object_lock) = in_range_object_weak.upgrade() {
                                    let mut in_range_object = in_range_object_lock.write().await;
                                    in_range_object
                                        .as_update_receiver_mut()
                                        .unwrap()
                                        .push_update_block(&mut buf.clone(), num_blocks);
                                }
                            }
                        }
                        valid_object.as_has_value_fields_mut().unwrap().clear_update_mask();
                    }

                    valid_object.as_update_receiver_mut().unwrap().process_pending_updates().await?;
                }
            }
        }
        Ok(())
    }

    pub async fn push_object<T>(&self, object_ref: Weak<RwLock<T>>)
    where
        T: GameObject + 'static,
    {
        let mut add_queue = self.add_queue.lock().await;
        add_queue.push(object_ref);
    }

    async fn process_add_queue(&self) -> Result<bool> {
        let cloned_add_queue = {
            let add_queue = &mut *self.add_queue.lock().await;
            let res = add_queue.clone();
            add_queue.clear();
            res
        };
        let has_any_added = !cloned_add_queue.is_empty();

        for to_add in cloned_add_queue.iter() {
            self.push_object_internal(to_add.clone()).await?;
        }

        Ok(has_any_added)
    }

    async fn push_object_internal(&self, object_ref: Weak<RwLock<dyn GameObject>>) -> Result<()> {
        if let Some(object_lock) = object_ref.upgrade() {
            {
                let object = object_lock.read().await;
                self.objects_on_map
                    .write()
                    .await
                    .insert(*object.as_map_object().get_guid(), object_ref.clone());
                if let Some(object) = object.as_world_object() {
                    let query_item = RStarTreeItem {
                        x: object.get_position().x,
                        y: object.get_position().y,
                        guid: *object.get_guid(),
                    };
                    self.query_tree.write().await.insert(query_item);
                }
            }

            self.update_in_range_set(object_ref).await?;

            object_lock.write().await.as_map_object_mut().on_pushed_to_map(self).await?;
        } else {
            bail!("Recieved a weak ref to a character that no longer exists");
        }

        Ok(())
    }

    async fn update_in_range_set(&self, object_ref: Weak<RwLock<dyn GameObject + 'static>>) -> Result<()> {
        if let Some(target_object_lock) = object_ref.upgrade() {
            if target_object_lock.read().await.as_world_object().is_none() {
                return Ok(());
            }
            let tree = self.query_tree.read().await;
            let within_range: Vec<&Guid> = {
                //Safe to unwrap because we checked it before
                let object = target_object_lock.read().await;
                let object = object.as_world_object().unwrap();
                let object_pos = [object.get_position().x, object.get_position().y];
                tree.locate_within_distance(object_pos, VISIBILITY_RANGE).map(|a| &a.guid).collect()
            };

            //Remove objects that we have in our in-range-list but that are no longer in range
            //according to the data tree
            {
                let mut destroyed_guids = vec![];
                {
                    let mut object = target_object_lock.write().await;
                    let object = object.as_world_object_mut().unwrap();
                    let in_range_list: Vec<Guid> = object.get_in_range_guids().iter().map(|a| *a.clone()).collect();
                    for guid in in_range_list.iter() {
                        if !within_range.contains(&guid) {
                            object.remove_in_range_object(guid)?;

                            destroyed_guids.push(guid.to_owned());
                        }
                    }
                }

                if let Some(character) = target_object_lock.read().await.as_character() {
                    for guid in destroyed_guids {
                        handlers::send_destroy_object(character, &guid, true).await?;
                    }
                }
            }

            let objects_on_map = self.objects_on_map.read().await;
            for guid in within_range.iter() {
                //Only read locks required here
                {
                    let target_object = target_object_lock.read().await;
                    let world_object = target_object.as_world_object().unwrap();
                    if *guid == world_object.get_guid() {
                        //skip ourselves
                        continue;
                    }

                    if world_object.is_in_range(guid) {
                        //skip if we already know this object in our range
                        continue;
                    }
                    trace!("New object in range! Guid: {}", guid);
                }

                if let Some(weak_ptr_to_lock) = objects_on_map.get(guid) {
                    if let Some(upgraded_lock_from_guid) = weak_ptr_to_lock.upgrade() {
                        let wants_updates = {
                            upgraded_lock_from_guid
                                .read()
                                .await
                                .as_world_object()
                                .and_then(|a| Some(a.wants_updates()))
                                .unwrap_or(false)
                        };
                        {
                            let mut write_obj = upgraded_lock_from_guid.write().await;

                            write_obj
                                .as_world_object_mut()
                                .unwrap()
                                .add_in_range_object(target_object_lock.read().await.as_world_object().unwrap().get_guid(), object_ref.clone())?;

                            if wants_updates {
                                let (block_count, mut buffer) = build_create_update_block_for_player(&*write_obj, &*target_object_lock.read().await)?;

                                if let Some(wants_updates) = write_obj.as_update_receiver_mut() {
                                    wants_updates.push_update_block(&mut buffer, block_count);
                                }
                            }
                        }
                        {
                            let mut write_target_object = target_object_lock.write().await;
                            let mut wants_updates = false;
                            if let Some(as_world_object) = write_target_object.as_world_object_mut() {
                                as_world_object.add_in_range_object(guid, weak_ptr_to_lock.clone())?;
                                wants_updates = as_world_object.wants_updates();
                            }
                            if wants_updates {
                                let (block_count, mut buffer) =
                                    build_create_update_block_for_player(&*write_target_object, &*upgraded_lock_from_guid.read().await)?;
                                if let Some(wants_updates) = write_target_object.as_update_receiver_mut() {
                                    wants_updates.push_update_block(&mut buffer, block_count);
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
                } else {
                    error!("Map manager had a GUID in its spacial querying tree that wasn't known in the objects on the map");
                    continue;
                }
            }
        }

        Ok(())
    }

    async fn rebuild_object_querying_tree(&self) -> Result<()> {
        let mut obj_list = vec![];
        let objects_on_map = self.objects_on_map.read().await;
        for (guid, obj_weak_ptr) in objects_on_map.iter() {
            if let Some(lock) = obj_weak_ptr.upgrade() {
                let map_obj = lock.read().await;
                if let Some(map_obj) = map_obj.as_world_object() {
                    obj_list.push(RStarTreeItem {
                        guid: *guid,
                        x: map_obj.get_position().x,
                        y: map_obj.get_position().y,
                    });
                }
            }
        }

        let mut query_tree = self.query_tree.write().await;
        *query_tree = RTree::bulk_load(obj_list);
        Ok(())
    }

    pub async fn remove_object_by_guid(&self, guid: &Guid) {
        let mut remove_queue = self.remove_queue.lock().await;
        remove_queue.push(*guid);
    }

    async fn process_remove_queue(&self) -> Result<bool> {
        let cloned_remove_queue = {
            let remove_queue = &mut *self.remove_queue.lock().await;
            let res = remove_queue.clone();
            remove_queue.clear();
            res
        };
        let any_to_remove = !cloned_remove_queue.is_empty();

        for to_remove in cloned_remove_queue.iter() {
            self.remove_object_by_guid_internal(to_remove).await?;
        }

        Ok(any_to_remove)
    }

    async fn remove_object_by_guid_internal(&self, guid: &Guid) -> Result<()> {
        if let Some(weak_removed_object) = {
            let mut objects_on_map = self.objects_on_map.write().await;
            objects_on_map.remove(guid)
        } {
            if let Some(removed_object_lock) = weak_removed_object.upgrade() {
                let mut removed_object = removed_object_lock.write().await;
                if let Some(removed_object) = removed_object.as_world_object_mut() {
                    let objects_on_map = self.objects_on_map.read().await;
                    for in_range_guid in removed_object.get_in_range_guids() {
                        if let Some(weak_in_range_object) = objects_on_map.get(in_range_guid) {
                            if let Some(in_range_object_lock) = weak_in_range_object.upgrade() {
                                let mut in_range_object = in_range_object_lock.write().await;
                                if let Some(character) = in_range_object.as_character() {
                                    handlers::send_destroy_object(character, guid, true).await?;
                                }
                                trace!("removed {} from range of {}", removed_object.get_guid(), in_range_guid);
                                if let Some(in_range_object) = in_range_object.as_world_object_mut() {
                                    in_range_object.remove_in_range_object(guid)?;
                                }
                            }
                        }
                    }
                    removed_object.clear_in_range_objects();
                }
            } else {
                //Failed to upgrade from a weakptr. This means the object is really gone, and we
                //can't access its in-range-list anymore. Bruteforce the removal from everything on
                //this map.
                let objects_on_map = self.objects_on_map.read().await;
                for object_weak in objects_on_map.values() {
                    if let Some(object_lock) = object_weak.upgrade() {
                        let mut object = object_lock.write().await;
                        if let Some(object) = object.as_world_object_mut() {
                            if object.is_in_range(guid) {
                                object.remove_in_range_object(guid)?;
                            }
                        }
                    }
                }
            }

            self.rebuild_object_querying_tree().await?;
        }
        Ok(())
    }
}
