use crate::character::Character;
use crate::client::Client;
use crate::prelude::*;
use async_std::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::map_manager::MapManager;
use super::map_object::MapObject;

type InstanceID = u32;
type MapID = u32;

pub struct InstanceManager {
    //Multiple instances are things like raids and dungeons which can spawn many times for
    //different groups
    multiple_instances: RwLock<HashMap<InstanceID, Arc<MapManager>>>,
    world_maps: RwLock<HashMap<MapID, Arc<MapManager>>>,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            multiple_instances: RwLock::new(HashMap::default()),
            world_maps: RwLock::new(HashMap::default()),
        }
    }

    pub async fn tick(&self, delta_time: f32) -> Result<()> {
        let instances_table = self.multiple_instances.read().await;
        for map in instances_table.values() {
            map.tick(delta_time).await?;
        }

        let world_maps = self.world_maps.read().await;
        for map in world_maps.values() {
            map.tick(delta_time).await?;
        }

        Ok(())
    }

    fn is_instance(&self, _map_id: MapID) -> bool {
        //TODO: implement based on DBC storage
        return false;
    }

    pub async fn get_or_create_map(&self, object: &impl MapObject, map_id: MapID) -> Result<Arc<MapManager>> {
        let map = if !self.is_instance(map_id) {
            Ok(self.world_maps.write().await.entry(map_id).or_insert(Arc::new(MapManager::new())).clone())
        } else if let Some(character) = object.as_character() {
            Ok(self.get_or_create_map_for_instance(character.instance_id).await)
        } else {
            Err(anyhow!("Not a valid map"))
        };

        map
    }

    pub async fn try_get_map_for_instance(&self, instance_id: InstanceID) -> Option<Arc<MapManager>> {
        if let Some(map) = self.multiple_instances.read().await.get(&instance_id) {
            Some(map.clone())
        } else {
            None
        }
    }

    pub async fn try_get_map_for_character(&self, character: &Character) -> Option<Arc<MapManager>> {
        self.get_or_create_map(character, character.map).await.ok()
    }

    async fn get_or_create_map_for_instance(&self, instance_id: InstanceID) -> Arc<MapManager> {
        self.multiple_instances
            .write()
            .await
            .entry(instance_id)
            .or_insert(Arc::new(MapManager::new()))
            .clone()
    }

    pub async fn handle_client_disconnected(&self, client: &Client) -> Result<()> {
        if let Some(character_lock) = &client.active_character {
            let active_character = character_lock.read().await;
            let map = self.try_get_map_for_character(&*active_character).await;
            if let Some(map) = map {
                map.remove_object_by_guid(active_character.get_guid()).await;
            }
        }

        Ok(())
    }
}
