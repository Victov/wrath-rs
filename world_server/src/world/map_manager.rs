use std::collections::HashMap;
use std::sync::Weak;

use super::map_object::MapObject;
use super::prelude::*;
use crate::prelude::*;
use async_std::sync::RwLock;
use rstar::{PointDistance, RTree, RTreeObject, AABB};

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
    objects_on_map: RwLock<HashMap<Guid, Weak<RwLock<dyn MapObjectWithValueFields>>>>,
    query_tree: RwLock<RTree<RStarTreeItem>>,
}

impl MapManager {
    pub fn new() -> Self {
        Self {
            objects_on_map: RwLock::new(HashMap::new()),
            query_tree: RwLock::new(RTree::new()),
        }
    }

    pub async fn try_get_object(&self, guid: &Guid) -> Option<Weak<RwLock<dyn MapObjectWithValueFields>>> {
        let map_objects = self.objects_on_map.read().await;
        map_objects.get(guid).and_then(|a| Some(a.clone()))
    }

    pub async fn tick(&self, delta_time: f32) -> Result<()> {
        let map_objects = self.objects_on_map.read().await;
        for map_object in (*map_objects).values() {
            if let Some(valid_object_lock) = map_object.upgrade() {
                let mut valid_object = valid_object_lock.write().await;

                //--------------------UGLY TEMP TESTING CODE -------------------------
                valid_object.advance_x(1.0f32 * delta_time);
                if valid_object.get_position().x > 10f32 {
                    valid_object.advance_x(-10f32);
                    let cur_displ_id = valid_object.get_unit_field_u32(UnitFields::Displayid)?;
                    let new_display_id = match cur_displ_id {
                        10000 => 19724,
                        19724 => 10000,
                        _ => 19724,
                    };
                    valid_object.set_unit_field_u32(UnitFields::Displayid, new_display_id)?;
                }
                //------------------END UGLY TEMP TESTING CODE -----------------------

                if valid_object.wants_updates() {
                    if valid_object.get_update_mask().has_any_bit() {
                        let (num_blocks, mut buf) = build_out_of_range_update_block_for_player(&*valid_object)?;
                        valid_object.clear_recently_removed_range_guids()?;
                        valid_object.as_update_receiver_mut().unwrap().push_update_block(&mut buf, num_blocks);

                        let (num_blocks, buf) = build_values_update_block(&*valid_object)?;
                        valid_object
                            .as_update_receiver_mut()
                            .unwrap()
                            .push_update_block(&mut buf.clone(), num_blocks);
                        let in_range_guids = valid_object.get_in_range_guids();
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
                        valid_object.clear_update_mask();
                    }

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
                self.objects_on_map.write().await.insert(object.get_guid().clone(), object_ref.clone());
                let query_item = RStarTreeItem {
                    x: object.get_position().x,
                    y: object.get_position().y,
                    guid: object.get_guid().clone(),
                };
                self.query_tree.write().await.insert(query_item);
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

        let tree = self.query_tree.read().await;
        let within_range = {
            let object = target_object_lock.read().await;
            let object_pos = [object.get_position().x, object.get_position().y];
            tree.locate_within_distance(object_pos, 200.0f32)
        };

        let objects_on_map = self.objects_on_map.read().await;
        for tree_item in within_range {
            let guid = tree_item.guid;
            //Only read locks required here
            {
                let target_object = target_object_lock.read().await;
                if &guid == target_object.get_guid() {
                    //skip ourselves
                    continue;
                }

                if target_object.is_in_range(&guid) {
                    //skip if we already know this object in our range
                    continue;
                }
                trace!("New object in range! Guid: {}", guid);
            }

            if let Some(weak_ptr_to_lock) = objects_on_map.get(&guid) {
                if let Some(upgraded_lock_from_guid) = weak_ptr_to_lock.upgrade() {
                    {
                        let mut write_obj = upgraded_lock_from_guid.write().await;
                        write_obj.add_in_range_object(target_object_lock.read().await.get_guid(), object_ref.clone())?;
                        if write_obj.wants_updates() {
                            let (block_count, mut buffer) = build_create_update_block_for_player(&*write_obj, &*target_object_lock.read().await)?;
                            if let Some(wants_updates) = write_obj.as_update_receiver_mut() {
                                wants_updates.push_update_block(&mut buffer, block_count);
                            }
                        }
                    }
                    {
                        let mut write_target_object = target_object_lock.write().await;
                        write_target_object.add_in_range_object(&guid, weak_ptr_to_lock.clone())?;
                        if write_target_object.wants_updates() {
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

        Ok(())
    }

    async fn rebuild_object_querying_tree(&self) -> Result<()> {
        let mut obj_list = vec![];
        let objects_on_map = self.objects_on_map.read().await;
        for (guid, obj_weak_ptr) in objects_on_map.iter() {
            if let Some(lock) = obj_weak_ptr.upgrade() {
                let map_obj = lock.read().await;
                obj_list.push(RStarTreeItem {
                    guid: *guid,
                    x: map_obj.get_position().x,
                    y: map_obj.get_position().y,
                });
            }
        }

        let mut query_tree = self.query_tree.write().await;
        *query_tree = RTree::bulk_load(obj_list);
        Ok(())
    }

    pub async fn remove_object_by_guid(&self, guid: &Guid) -> Result<()> {
        if let Some(weak_removed_object) = {
            let mut objects_on_map = self.objects_on_map.write().await;
            objects_on_map.remove(guid)
        } {
            if let Some(removed_object_lock) = weak_removed_object.upgrade() {
                let removed_object = removed_object_lock.read().await;
                let objects_on_map = self.objects_on_map.read().await;
                for in_range_guid in removed_object.get_in_range_guids() {
                    if let Some(weak_in_range_object) = objects_on_map.get(in_range_guid) {
                        if let Some(in_range_object_lock) = weak_in_range_object.upgrade() {
                            let mut in_range_object = in_range_object_lock.write().await;
                            if let Some(character) = in_range_object.as_character() {
                                handlers::send_destroy_object(character, guid, true).await?;
                            }
                            trace!("removed {} from range of {}", removed_object.get_guid(), in_range_guid);
                            in_range_object.remove_in_range_object(guid)?;
                        }
                    }
                }
            }

            self.rebuild_object_querying_tree().await?;
        }
        Ok(())
    }
}
