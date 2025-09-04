use super::{
    instance_manager::MapID,
    prelude::{build_create_update_block_for_player, build_out_of_range_update_block_for_player, build_values_update_block},
};
use std::collections::HashSet;

use super::prelude::GameObject;
use crate::world::update_builder::ReceiveUpdates;
use crate::{
    character::{character_manager::CharacterManager, Character},
    prelude::*,
};
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

    characters_on_map: HashSet<Guid>,
    characters_query_tree: RTree<RStarTreeItem>,
    add_queue: Vec<Guid>,
    remove_queue: Vec<Guid>,
}

impl MapManager {
    pub fn new(id: MapID) -> Self {
        info!("spawned new map with id {}", id);
        Self {
            id,
            characters_on_map: HashSet::new(),
            characters_query_tree: RTree::new(),
            add_queue: Vec::new(),
            remove_queue: Vec::new(),
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Map {} shutting down", self.id);
        Ok(())
    }

    pub async fn should_shutdown(&self) -> bool {
        self.characters_on_map.is_empty()
    }

    pub async fn tick(&mut self, _delta_time: f32, character_manager: &mut CharacterManager) -> Result<()> {
        self.rebuild_object_querying_tree(character_manager)?;
        let any_removed = self.process_remove_queue(character_manager).await?;
        let any_added = self.process_add_queue(character_manager)?;

        for guid in self.characters_on_map.iter().copied() {
            self.update_in_range_set(guid, character_manager).await?;

            let has_any_update_bit = if let UpdateMask::Player(update_mask) = character_manager.get_character(guid)?.get_update_mask() {
                update_mask.has_any_dirty_fields()
            } else {
                bail!("any other type not supported");
            };

            let has_something_recently_removed = character_manager.get_character(guid)?.get_recently_removed_range_guids().is_empty();

            if has_any_update_bit || any_removed || any_added || has_something_recently_removed {
                {
                    let character = character_manager.get_character_mut(guid)?;
                    if let Some(out_of_range_update) = build_out_of_range_update_block_for_player(character) {
                        character.clear_recently_removed_range_guids();
                        character.push_object_update(out_of_range_update);
                    }
                }

                if has_any_update_bit {
                    let values_update = {
                        let character = character_manager.get_character_mut(guid)?;
                        let values_update = build_values_update_block(character)?;
                        character.push_object_update(values_update.clone());
                        values_update
                    };

                    for in_range_guid in character_manager.get_character_mut(guid)?.get_in_range_characters().to_vec() {
                        let in_range_character = character_manager.get_character_mut(in_range_guid)?;
                        if let Some(update_receiver) = in_range_character.as_update_receiver_mut() {
                            update_receiver.push_object_update(values_update.clone());
                        }
                    }

                    let character = character_manager.get_character_mut(guid)?;
                    character.clear_update_mask_header();
                }
            }
            let character = character_manager.get_character_mut(guid)?;
            character.process_pending_updates().await?;
        }
        Ok(())
    }

    pub fn push_character(&mut self, character: &Character) {
        self.add_queue.push(character.get_guid());
    }

    pub fn find_character(&self, guid: Guid) -> bool {
        self.characters_on_map.contains(&guid)
    }

    fn process_add_queue(&mut self, character_manager: &mut CharacterManager) -> Result<bool> {
        let has_any_added = !self.add_queue.is_empty();

        for to_add in self.add_queue.clone() {
            self.push_character_internal(to_add, character_manager)?;
        }
        self.add_queue.clear();

        Ok(has_any_added)
    }

    fn push_character_internal(&mut self, guid: Guid, character_manager: &mut CharacterManager) -> Result<()> {
        let character = character_manager.get_character(guid)?;
        let position = character.get_position().unwrap();
        self.characters_on_map.insert(guid);
        let query_item = RStarTreeItem {
            x: position.position.x,
            y: position.position.y,
            guid,
        };
        self.characters_query_tree.insert(query_item);

        character_manager.get_character_mut(guid)?.on_pushed_to_map(self)?;
        Ok(())
    }

    async fn update_in_range_set(&self, guid: Guid, character_manager: &mut CharacterManager) -> Result<()> {
        //Check if this object even has positional data
        if character_manager.get_character(guid)?.get_position().is_none() {
            return Ok(());
        }
        let tree = &self.characters_query_tree;
        let within_range: Vec<Guid> = {
            //Safe to unwrap because we checked it before
            let character = character_manager.get_character(guid)?;
            let position = character.get_position().unwrap().position;
            tree.locate_within_distance([position.x, position.y], VISIBILITY_RANGE)
                .map(|a| a.guid)
                .collect()
        };

        //Remove objects that we have in our in-range-list but that are no longer in range
        //according to the data tree
        {
            let mut destroyed_guids = vec![];
            {
                let character = character_manager.get_character_mut(guid)?;
                let in_range_list: Vec<Guid> = character.get_in_range_guids();
                for guid in in_range_list {
                    if !within_range.contains(&guid) {
                        character.remove_in_range_object(guid)?;

                        destroyed_guids.push(guid);
                    }
                }
            }

            for guid in destroyed_guids {
                let character = character_manager.get_character(guid)?;
                handlers::send_destroy_object(character, guid, false).await?;
            }
        }

        for in_range_guid in within_range {
            {
                if in_range_guid == guid {
                    //skip ourselves
                    continue;
                }

                let character = character_manager.get_character(guid)?;
                if character.is_in_range(in_range_guid) {
                    //skip if we already know this object in our range
                    continue;
                }
            }

            trace!("New object in range! Guid: {}", in_range_guid);

            {
                let other_character = character_manager.get_character_mut(in_range_guid)?;
                other_character.add_in_range_character(guid)?;
            }
            {
                let other_character = character_manager.get_character(in_range_guid)?;
                let character = character_manager.get_character(guid)?;
                let create_block = build_create_update_block_for_player(other_character, character)?;
                let other_character = character_manager.get_character_mut(in_range_guid)?;
                other_character.push_object_update(create_block);
            }
            {
                let character = character_manager.get_character_mut(guid)?;
                character.add_in_range_character(in_range_guid)?;
            }
            {
                let other_character = character_manager.get_character(in_range_guid)?;
                let character = character_manager.get_character(guid)?;
                let create_block = build_create_update_block_for_player(character, other_character)?;
                let character = character_manager.get_character_mut(guid)?;
                character.push_object_update(create_block);
            }
        }

        Ok(())
    }

    fn rebuild_object_querying_tree(&mut self, character_manager: &CharacterManager) -> Result<()> {
        let mut obj_list = vec![];
        for guid in self.characters_on_map.iter().copied() {
            let character = character_manager.get_character(guid)?;
            if let Some(position) = character.get_position() {
                obj_list.push(RStarTreeItem {
                    guid,
                    x: position.position.x,
                    y: position.position.y,
                });
            }
        }

        self.characters_query_tree = RTree::bulk_load(obj_list);
        Ok(())
    }

    pub fn remove_object_by_guid(&mut self, guid: Guid) {
        self.remove_queue.push(guid);
    }

    async fn process_remove_queue(&mut self, character_manager: &mut CharacterManager) -> Result<bool> {
        let any_to_remove = !self.remove_queue.is_empty();
        for to_remove in self.remove_queue.clone() {
            self.remove_object_by_guid_internal(to_remove, character_manager).await?;
        }
        self.remove_queue.clear();
        Ok(any_to_remove)
    }

    async fn remove_object_by_guid_internal(&mut self, guid: Guid, character_manager: &mut CharacterManager) -> Result<()> {
        if self.characters_on_map.remove(&guid) {
            if character_manager.find_character(guid).is_some() {
                let in_range_guids = {
                    let removed_character = character_manager.get_character(guid)?;
                    removed_character.get_in_range_guids().to_vec()
                };

                for &in_range_guid in in_range_guids.iter().filter_map(|g| self.characters_on_map.get(g)) {
                    let in_range_character = character_manager.get_character_mut(in_range_guid)?;
                    handlers::send_destroy_object(in_range_character, guid, false).await?;
                    trace!("removed {} from range of {}", guid, in_range_guid);
                    in_range_character.remove_in_range_object(guid)?;
                }

                let removed_character = character_manager.get_character_mut(guid)?;
                removed_character.clear_in_range_objects();
            } else {
                //Failed to find character. This means the object is really gone, and we
                //can't access its in-range-list anymore. Bruteforce the removal from everything on
                //this map.
                for &character_guid in self.characters_on_map.iter() {
                    let character = character_manager.get_character_mut(character_guid)?;
                    if character.is_in_range(guid) {
                        character.remove_in_range_object(guid)?;
                    }
                }
            }

            self.rebuild_object_querying_tree(character_manager)?;
        }
        Ok(())
    }
}
