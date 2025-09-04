use crate::character::character_manager::CharacterManager;
use crate::character::Character;
use crate::client::Client;
use crate::prelude::*;
use std::collections::HashMap;
use wow_world_messages::wrath::Map;

use super::map_manager::MapManager;
use super::prelude::GameObject;

pub type InstanceID = u32;
pub type MapID = u32;

#[derive(Default)]
pub struct InstanceManager {
    //Multiple instances are things like raids and dungeons which can spawn many times for
    //different groups
    multiple_instances: HashMap<InstanceID, MapManager>,
    world_maps: HashMap<MapID, MapManager>,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            multiple_instances: HashMap::default(),
            world_maps: HashMap::default(),
        }
    }

    pub async fn tick(&mut self, character_manager: &mut CharacterManager, delta_time: f32) -> Result<()> {
        Self::tick_maps::<MapID>(&mut self.world_maps, character_manager, delta_time).await?;
        Self::tick_maps::<InstanceID>(&mut self.multiple_instances, character_manager, delta_time).await?;
        Self::cleanup_maps::<MapID>(&mut self.world_maps).await?;
        Self::cleanup_maps::<InstanceID>(&mut self.multiple_instances).await?;

        Ok(())
    }

    async fn tick_maps<T: PartialEq + Clone>(
        maps: &mut HashMap<T, MapManager>,
        character_manager: &mut CharacterManager,
        delta_time: f32,
    ) -> Result<()> {
        for map in maps.values_mut() {
            map.tick(delta_time, character_manager).await?;
        }
        Ok(())
    }

    async fn cleanup_maps<T: PartialEq + Clone>(maps: &mut HashMap<T, MapManager>) -> Result<()> {
        let mut to_cleanup = Vec::new();

        for (id, map) in maps.iter() {
            if map.should_shutdown().await {
                map.shutdown().await?;
                to_cleanup.push(id.clone());
            }
        }

        if !to_cleanup.is_empty() {
            maps.retain(|k, _| !to_cleanup.contains(k));
        }

        Ok(())
    }

    fn is_instance(&self, _map_id: Map) -> bool {
        //TODO: implement based on DBC storage
        false
    }

    pub async fn get_or_create_map(&mut self, object: &impl GameObject, map: Map) -> Result<&mut MapManager> {
        let map = if !self.is_instance(map) {
            Ok(self.world_maps.entry(map.as_int()).or_insert_with(|| MapManager::new(map.as_int())))
        } else if let Some(character) = object.as_character() {
            Ok(self.get_or_create_map_for_instance(map, character.instance_id).await)
        } else {
            Err(anyhow!("Not a valid map"))
        };

        map
    }

    pub async fn try_get_map_for_instance(&self, instance_id: InstanceID) -> Option<&MapManager> {
        self.multiple_instances.get(&instance_id)
    }

    pub fn try_get_map_for_character(&self, character: &Character) -> Option<&MapManager> {
        if !self.is_instance(character.map) {
            self.world_maps.get(&character.map.as_int())
        } else {
            self.multiple_instances.get(&character.instance_id)
        }
    }

    pub fn try_get_map_for_character_mut(&mut self, character: &Character) -> Option<&mut MapManager> {
        if !self.is_instance(character.map) {
            self.world_maps.get_mut(&character.map.as_int())
        } else {
            self.multiple_instances.get_mut(&character.instance_id)
        }
    }

    async fn get_or_create_map_for_instance(&mut self, map: Map, instance_id: InstanceID) -> &mut MapManager {
        self.multiple_instances.entry(instance_id).or_insert(MapManager::new(map.as_int()))
    }

    pub async fn handle_client_disconnected(&mut self, client: &Client, character_manager: &CharacterManager) -> Result<()> {
        if let Some(guid) = client.data.active_character {
            let character = character_manager.get_character(guid)?;
            let map = self.try_get_map_for_character_mut(character);
            if let Some(map) = map {
                map.remove_object_by_guid(guid);
            }
        }

        Ok(())
    }
}
