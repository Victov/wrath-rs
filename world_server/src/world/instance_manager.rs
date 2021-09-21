use crate::client::Client;
use crate::prelude::*;
use async_std::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::map_manager::MapManager;

type InstanceID = u32;

pub struct InstanceManager {
    //Multiple instances are things like raids and dungeons which can spawn many times for
    //different groups
    multiple_instances: RwLock<HashMap<InstanceID, Arc<MapManager>>>,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            multiple_instances: RwLock::new(HashMap::default()),
        }
    }

    pub async fn tick(&self, delta_time: f32) -> Result<()> {
        let maps_table = self.multiple_instances.read().await;
        for map in maps_table.values() {
            map.tick(delta_time).await?;
        }

        Ok(())
    }

    pub async fn get_map_for_instance(&self, instance_id: InstanceID) -> Arc<MapManager> {
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
            let character_instance_id = active_character.instance_id;

            let mul_instances = self.multiple_instances.read().await;
            if let Some(map_mgr) = mul_instances.get(&character_instance_id) {
                use super::map_object::MapObject;
                map_mgr.remove_object_by_guid(active_character.get_guid()).await?;
            } else {
                warn!("Handling disconnent, but no known map manager");
            }
        }

        Ok(())
    }
}
