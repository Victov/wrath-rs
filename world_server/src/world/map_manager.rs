use super::{
    instance_manager::MapID,
    prelude::{build_create_update_block_for_player, build_out_of_range_update_block_for_player, build_values_update_block},
};
use std::collections::HashMap;
use std::sync::Weak;

use super::prelude::GameObject;
use crate::prelude::*;
use async_std::sync::{Mutex, RwLock};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use wow_world_messages::wrath::UpdateMask;

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

    pub async fn try_get_object(&self, guid: Guid) -> Option<Weak<RwLock<dyn GameObject>>> {
        let map_objects = self.objects_on_map.read().await;
        map_objects.get(&guid).cloned()
    }

    pub async fn tick(&self, _delta_time: f32) -> Result<()> {
        self.rebuild_object_querying_tree().await?;

        let any_removed = self.process_remove_queue().await?;
        let any_added = self.process_add_queue().await?;

        let map_objects = self.objects_on_map.read().await;
        for weak_locked_map_object in (*map_objects).values() {
            self.update_in_range_set(weak_locked_map_object.clone()).await?;

            if let Some(locked_map_object) = weak_locked_map_object.upgrade() {
                let mut map_object = locked_map_object.write().await;
                let wants_to_receive_update_blocks = map_object.as_update_receiver().is_some();
                assert_eq!(
                    map_object.as_update_receiver().is_some(),
                    map_object.as_update_receiver_mut().is_some(),
                    "Implementing one without the other, this will cause problems on an upwrap later"
                );

                if wants_to_receive_update_blocks {
                    let has_any_update_bit = if let UpdateMask::Player(update_mask) = map_object.get_update_mask() {
                        update_mask.has_any_header_set()
                    } else {
                        bail!("any other type not supported");
                    };

                    let has_something_recently_removed = map_object.get_recently_removed_range_guids().is_empty();

                    if has_any_update_bit || any_removed || any_added || has_something_recently_removed {
                        if let Some(out_of_range_update) = build_out_of_range_update_block_for_player(&*map_object) {
                            map_object.clear_recently_removed_range_guids();
                            map_object.as_update_receiver_mut().unwrap().push_object_update(out_of_range_update);
                        }

                        let values_update = build_values_update_block(&*map_object)?;

                        map_object.as_update_receiver_mut().unwrap().push_object_update(values_update.clone());

                        let in_range_guids = map_object.get_in_range_guids();
                        for in_range_object_lock in in_range_guids
                            .iter()
                            .filter_map(|guid| map_objects.get(guid))
                            .filter_map(|weak| weak.upgrade())
                        {
                            let mut in_range_object = in_range_object_lock.write().await;
                            if let Some(update_receiver) = in_range_object.as_update_receiver_mut() {
                                update_receiver.push_object_update(values_update.clone());
                            }
                        }
                        map_object.clear_update_mask_header();
                    }

                    map_object.as_update_receiver_mut().unwrap().process_pending_updates().await?;
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
                if let Some(position) = object.get_position() {
                    self.objects_on_map.write().await.insert(object.get_guid(), object_ref.clone());
                    let query_item = RStarTreeItem {
                        x: position.position.x,
                        y: position.position.y,
                        guid: object.get_guid(),
                    };
                    self.query_tree.write().await.insert(query_item);
                }
            }

            self.update_in_range_set(object_ref).await?;

            object_lock.write().await.on_pushed_to_map(self).await?;
        } else {
            bail!("Recieved a weak ref to a character that no longer exists");
        }

        Ok(())
    }

    async fn update_in_range_set(&self, weak_object_lock: Weak<RwLock<dyn GameObject + 'static>>) -> Result<()> {
        if let Some(object_lock) = weak_object_lock.upgrade() {
            //Check if this object even has positional data
            if object_lock.read().await.get_position().is_none() {
                return Ok(());
            }
            let tree = self.query_tree.read().await;
            let within_range: Vec<Guid> = {
                //Safe to unwrap because we checked it before
                let object = object_lock.read().await;
                let position = object.get_position().unwrap().position;
                tree.locate_within_distance([position.x, position.y], VISIBILITY_RANGE)
                    .map(|a| a.guid)
                    .collect()
            };

            //Remove objects that we have in our in-range-list but that are no longer in range
            //according to the data tree
            {
                let mut destroyed_guids = vec![];
                {
                    let mut object = object_lock.write().await;
                    let in_range_list: Vec<Guid> = object.get_in_range_guids();
                    for guid in in_range_list {
                        if !within_range.contains(&guid) {
                            object.remove_in_range_object(guid)?;

                            destroyed_guids.push(guid);
                        }
                    }
                }

                if let Some(character) = object_lock.read().await.as_character() {
                    for guid in destroyed_guids {
                        handlers::send_destroy_object(character, guid, true).await?;
                    }
                }
            }

            let objects_on_map = self.objects_on_map.read().await;
            for guid in within_range {
                //Only read locks required here
                {
                    let target_object = object_lock.read().await;
                    if guid == target_object.get_guid() {
                        //skip ourselves
                        continue;
                    }

                    if target_object.is_in_range(guid) {
                        //skip if we already know this object in our range
                        continue;
                    }
                    trace!("New object in range! Guid: {}", guid);
                }

                if let Some(lock_from_guid) = objects_on_map.get(&guid).and_then(|weak| weak.upgrade()) {
                    let wants_updates = { lock_from_guid.read().await.as_update_receiver().is_some() };
                    {
                        let mut write_obj = lock_from_guid.write().await;

                        write_obj.add_in_range_object(object_lock.read().await.get_guid(), weak_object_lock.clone())?;

                        if wants_updates {
                            let update_block = build_create_update_block_for_player(&*write_obj, &*object_lock.read().await)?;
                            write_obj.as_update_receiver_mut().unwrap().push_object_update(update_block);
                        }
                    }
                    let mut object = object_lock.write().await;
                    object.add_in_range_object(guid, weak_object_lock.clone())?;
                    let wants_updates = object.as_update_receiver().is_some();
                    if wants_updates {
                        let update_block = build_create_update_block_for_player(&*object, &*object_lock.read().await)?;
                        object.as_update_receiver_mut().unwrap().push_object_update(update_block);
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
        for (guid, weak_lock) in objects_on_map.iter() {
            if let Some(lock) = weak_lock.upgrade() {
                let map_obj = lock.read().await;
                if let Some(position) = map_obj.get_position() {
                    obj_list.push(RStarTreeItem {
                        guid: *guid,
                        x: position.position.x,
                        y: position.position.y,
                    });
                }
            }
        }

        let mut query_tree = self.query_tree.write().await;
        *query_tree = RTree::bulk_load(obj_list);
        Ok(())
    }

    pub async fn remove_object_by_guid(&self, guid: Guid) {
        let mut remove_queue = self.remove_queue.lock().await;
        remove_queue.push(guid);
    }

    async fn process_remove_queue(&self) -> Result<bool> {
        let cloned_remove_queue = {
            let remove_queue = &mut *self.remove_queue.lock().await;
            let res = remove_queue.clone();
            remove_queue.clear();
            res
        };
        let any_to_remove = !cloned_remove_queue.is_empty();

        for &to_remove in cloned_remove_queue.iter() {
            self.remove_object_by_guid_internal(to_remove).await?;
        }

        Ok(any_to_remove)
    }

    async fn remove_object_by_guid_internal(&self, guid: Guid) -> Result<()> {
        if let Some(weak_removed_object) = {
            let mut objects_on_map = self.objects_on_map.write().await;
            objects_on_map.remove(&guid)
        } {
            if let Some(removed_object_lock) = weak_removed_object.upgrade() {
                let mut removed_object = removed_object_lock.write().await;
                let objects_on_map = self.objects_on_map.read().await;
                for in_range_object_lock in removed_object
                    .get_in_range_guids()
                    .iter()
                    .filter_map(|g| objects_on_map.get(g).and_then(|weak| weak.upgrade()))
                {
                    let mut in_range_object = in_range_object_lock.write().await;
                    if let Some(character) = in_range_object.as_character() {
                        handlers::send_destroy_object(character, guid, true).await?;
                    }
                    trace!("removed {} from range of {}", removed_object.get_guid(), in_range_object.get_guid());
                    in_range_object.remove_in_range_object(guid)?;
                }
                removed_object.clear_in_range_objects();
            } else {
                //Failed to upgrade from a weakptr. This means the object is really gone, and we
                //can't access its in-range-list anymore. Bruteforce the removal from everything on
                //this map.
                let objects_on_map = self.objects_on_map.read().await;
                for object_lock in objects_on_map.values().filter_map(|w| w.upgrade()) {
                    let mut object = object_lock.write().await;
                    if object.is_in_range(guid) {
                        object.remove_in_range_object(guid)?;
                    }
                }
            }

            self.rebuild_object_querying_tree().await?;
        }
        Ok(())
    }
}
