use crate::character::Character;
use crate::client::Client;
use crate::prelude::*;
use async_std::sync::{RwLock, RwLockUpgradableReadGuard};
use std::collections::HashMap;
use std::sync::Arc;

use super::map_manager::MapManager;
use super::map_object::MapObject;

pub type InstanceID = u32;
pub type MapID = u32;

#[derive(Default)]
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
        self.tick_maps::<MapID>(&self.world_maps, delta_time).await?;
        self.tick_maps::<InstanceID>(&self.multiple_instances, delta_time).await?;
        self.cleanup_maps::<MapID>(&self.world_maps).await?;
        self.cleanup_maps::<InstanceID>(&self.multiple_instances).await?;

        Ok(())
    }

    async fn tick_maps<T: PartialEq + Clone>(&self, list: &RwLock<HashMap<T, Arc<MapManager>>>, delta_time: f32) -> Result<()> {
        let maps = list.read().await;
        for map in maps.values() {
            map.tick(delta_time).await?;
        }
        Ok(())
    }

    async fn cleanup_maps<T: PartialEq + Clone>(&self, list: &RwLock<HashMap<T, Arc<MapManager>>>) -> Result<()> {
        let mut to_cleanup = Vec::new();

        let maps = list.upgradable_read().await;
        {
            for (id, map) in maps.iter() {
                if map.should_shutdown().await {
                    map.shutdown().await?;
                    to_cleanup.push(id.clone());
                }
            }
        }

        if !to_cleanup.is_empty() {
            let mut maps = RwLockUpgradableReadGuard::upgrade(maps).await;
            maps.retain(|k, _| !to_cleanup.contains(k));
        }

        Ok(())
    }

    fn is_instance(&self, _map_id: MapID) -> bool {
        //TODO: implement based on DBC storage
        false
    }

    pub async fn get_or_create_map(&self, object: &impl MapObject, map_id: MapID) -> Result<Arc<MapManager>> {
        let map = if !self.is_instance(map_id) {
            Ok(self
                .world_maps
                .write()
                .await
                .entry(map_id)
                .or_insert(Arc::new(MapManager::new(map_id)))
                .clone())
        } else if let Some(character) = object.as_character() {
            Ok(self.get_or_create_map_for_instance(map_id, character.instance_id).await)
        } else {
            Err(anyhow!("Not a valid map"))
        };

        map
    }

    pub async fn try_get_map_for_instance(&self, instance_id: InstanceID) -> Option<Arc<MapManager>> {
        self.multiple_instances.read().await.get(&instance_id).cloned()
    }

    pub async fn try_get_map_for_character(&self, character: &Character) -> Option<Arc<MapManager>> {
        if !self.is_instance(character.map) {
            self.world_maps.read().await.get(&character.map).cloned()
        } else {
            self.multiple_instances.read().await.get(&character.instance_id).cloned()
        }
    }

    async fn get_or_create_map_for_instance(&self, map_id: MapID, instance_id: InstanceID) -> Arc<MapManager> {
        self.multiple_instances
            .write()
            .await
            .entry(instance_id)
            .or_insert(Arc::new(MapManager::new(map_id)))
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
