use super::prelude::*;
use crate::prelude::*;

#[async_trait::async_trait]
pub trait ReceiveUpdates {
    fn push_object_update(&mut self, object_update: wow_world_messages::wrath::Object);
    fn get_object_updates(&self) -> &Vec<wow_world_messages::wrath::Object>;
    fn clear_object_updates(&mut self);
    async fn process_pending_updates(&mut self) -> Result<()>;
}

pub fn build_create_update_block_for_player(player: &dyn GameObject, object: &dyn GameObject) -> Result<wow_world_messages::wrath::Object> {
    use wow_world_messages::wrath::{
        MovementBlock, MovementBlock_MovementFlags, MovementBlock_UpdateFlag, MovementBlock_UpdateFlag_Living, Object, Object_UpdateType,
    };

    assert!(object.as_character().is_some(), "Only characters currently supported");

    let object_guid = object.get_guid();
    let player_guid = player.get_guid();
    let creating_self = player_guid == object_guid;

    let movement_info = object.get_movement_info();

    //TODO: convert movement_info.flags into movement_flags.
    //They should be identical but since they are different types,
    //I can't just pass them along so easily. Maybe write From traits for conversion?
    let movement_flags = MovementBlock_MovementFlags::empty();

    let mut update_flag = MovementBlock_UpdateFlag::empty()
        .set_LIVING(MovementBlock_UpdateFlag_Living::Living {
            backwards_running_speed: 4.5,
            backwards_swimming_speed: 0.0,
            extra_flags: movement_info.extra_flags,
            fall_time: movement_info.fall_time,
            flags: movement_flags,
            flight_speed: 0.0,
            backwards_flight_speed: 0.0,
            living_orientation: movement_info.orientation,
            living_position: movement_info.position,
            pitch_rate: 0.0,
            running_speed: 7.0,
            swimming_speed: 0.0,
            timestamp: movement_info.timestamp,
            turn_rate: std::f32::consts::PI,
            walking_speed: 1.0,
        })
        .set_HIGH_GUID(wow_world_messages::wrath::MovementBlock_UpdateFlag_HighGuid {
            unknown0: if creating_self { 0x2F } else { 0x08 },
        })/*
        .set_LOW_GUID(wow_world_messages::wrath::MovementBlock_UpdateFlag_LowGuid {
            unknown1: object_guid.guid() as u32,
        })*/;

    if creating_self {
        update_flag = update_flag.set_SELF()
    }

    //Copy the update mask and mark every field dirty, so that we send everything we need to know
    let mut all_dirty_update_mask = object.get_update_mask();
    match all_dirty_update_mask {
        wow_world_messages::wrath::UpdateMask::Player(ref mut inner) => inner.mark_fully_dirty(),
        _ => unimplemented!(),
    }

    let update_type = if creating_self {
        Object_UpdateType::CreateObject2 {
            guid3: object_guid,
            mask2: all_dirty_update_mask,
            movement2: MovementBlock { update_flag },
            object_type: object.get_type(),
        }
    } else {
        Object_UpdateType::CreateObject {
            guid3: object_guid,
            mask2: all_dirty_update_mask,
            movement2: MovementBlock { update_flag },
            object_type: object.get_type(),
        }
    };

    Ok(Object { update_type })
}

pub fn build_out_of_range_update_block_for_player(player: &dyn GameObject) -> Option<wow_world_messages::wrath::Object> {
    use wow_world_messages::wrath::{Object, Object_UpdateType};

    let out_of_range_guids = player.get_recently_removed_range_guids();
    if out_of_range_guids.is_empty() {
        None
    } else {
        Some(Object {
            update_type: Object_UpdateType::OutOfRangeObjects {
                guids: out_of_range_guids.to_vec(),
            },
        })
    }
}

pub fn build_values_update_block(object: &dyn GameObject) -> Result<wow_world_messages::wrath::Object> {
    use wow_world_messages::wrath::{Object, Object_UpdateType};

    Ok(Object {
        update_type: Object_UpdateType::Values {
            guid1: object.get_guid(),
            mask1: object.get_update_mask(),
        },
    })
}
