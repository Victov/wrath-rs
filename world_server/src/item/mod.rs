use crate::{
    character::Character,
    prelude::*,
    world::value_fields::{ItemFields, UpdateMask, ValueFieldsRaw},
    world::{item_value_helpers::ItemValueHelpers, prelude::*},
};
use async_std::sync::RwLock;
use std::sync::Arc;

const NUM_ITEM_FIELDS: usize = ItemFields::End as usize;

pub struct Item {
    pub id: u32,
    guid: Guid,
    owner: Arc<RwLock<Character>>,
    item_value_fields: [u32; NUM_ITEM_FIELDS],
    changed_update_mask: UpdateMask,
}

impl Item {
    pub async fn new(id: u32, owner: &Character, world: &World) -> Result<Self> {
        let item_template = world.get_realm_database().get_item_template(id).await?;
        use rand::RngCore;
        let low_guid: u32 = rand::thread_rng().next_u32();

        let guid = Guid::new(low_guid, HighGuid::ItemOrContainer);

        let mut result = Self {
            id,
            owner: owner.try_get_self_arc().await?,
            guid,
            item_value_fields: [0; NUM_ITEM_FIELDS],
            changed_update_mask: UpdateMask::new(NUM_ITEM_FIELDS),
        };

        result.set_owner(owner.get_guid())?;
        result.set_contained(&guid)?;
        result.set_stack_count(1)?;
        result.set_durability(item_template.max_durability as u32)?;
        result.set_max_durability(item_template.max_durability as u32)?;
        result.set_duration(item_template.duration)?;

        Ok(result)
    }
}

impl ValueFieldsRaw for Item {
    fn set_field_u32(&mut self, field: usize, value: u32) -> Result<()> {
        if field > self.item_value_fields.len() {
            bail!("Out-of-range item field being set")
        }
        self.item_value_fields[field] = value;
        self.changed_update_mask.set_bit(field, true)?;
        Ok(())
    }

    fn get_field_u32(&self, field: usize) -> Result<u32> {
        if field > self.item_value_fields.len() {
            bail!("Out-of-range item field being accessed");
        }
        Ok(self.item_value_fields[field])
    }

    fn get_num_value_fields(&self) -> usize {
        NUM_ITEM_FIELDS
    }

    fn clear_update_mask(&mut self) {
        self.changed_update_mask.clear();
    }

    fn get_update_mask(&self) -> &UpdateMask {
        &self.changed_update_mask
    }
}

impl GameObject for Item {
    fn as_map_object(&self) -> &dyn MapObject {
        self
    }

    fn as_map_object_mut(&mut self) -> &mut dyn MapObject {
        self
    }

    fn as_character(&self) -> Option<&Character> {
        None
    }

    fn as_has_value_fields(&self) -> Option<&dyn HasValueFields> {
        Some(self)
    }

    fn as_has_value_fields_mut(&mut self) -> Option<&mut dyn HasValueFields> {
        Some(self)
    }

    fn as_update_receiver_mut(&mut self) -> Option<&mut dyn ReceiveUpdates> {
        None
    }

    fn as_update_receiver(&self) -> Option<&dyn ReceiveUpdates> {
        None
    }

    fn as_world_object(&self) -> Option<&dyn WorldObject> {
        None
    }

    fn as_world_object_mut(&mut self) -> Option<&mut dyn WorldObject> {
        None
    }
}

#[async_trait::async_trait]
impl MapObject for Item {
    fn get_guid(&self) -> &Guid {
        &self.guid
    }

    fn get_type(&self) -> updates::ObjectType {
        ObjectType::Item
    }

    async fn on_pushed_to_map(&mut self, _map_manager: &MapManager) -> Result<()> {
        info!("item was pushed to map");
        //let owner = self.owner.read().await;

        //let (block_count, mut update_data) = build_create_update_block_for_player(&owner, self)?;
        //self.owner.read().await.push_update_block(&mut update_data, block_count);
        Ok(())
    }
}
