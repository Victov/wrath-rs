//see https://github.com/arcemu/arcemu/blob/master/src/world/Game/Entities/Update/UpdateBuilder.cpp
//for reference

use crate::data::WriteMovementInfo;
use crate::prelude::*;
use podio::{LittleEndian, WritePodExt};
use std::io::Cursor;

use super::super::constants::updates::*;
use super::prelude::{HasValueFields, MapObject, ObjectFields, UpdateMask};

#[async_trait::async_trait]
pub trait ReceiveUpdates {
    fn push_update_block(&mut self, data: &mut Vec<u8>, block_count: u32);
    fn get_update_blocks(&self) -> (u32, &Vec<u8>);
    fn clear_update_blocks(&mut self);
    async fn process_pending_updates(&mut self) -> Result<()>;
}

pub trait MapObjectWithValueFields: MapObject + HasValueFields {}
impl<T> MapObjectWithValueFields for T where T: MapObject + HasValueFields {}

pub fn build_create_update_block_for_player(player: &dyn MapObjectWithValueFields, object: &dyn MapObjectWithValueFields) -> Result<(u32, Vec<u8>)> {
    let outputbuf = Vec::<u8>::new();
    let mut update_type = ObjectUpdateType::CreateObject as u8;
    let mut writer = Cursor::new(outputbuf);
    let mut block_count = 0;

    let mut flags: u16 = match object.get_type() {
        ObjectType::Item => ObjectUpdateFlags::HighGuid as u16,
        ObjectType::Container => ObjectUpdateFlags::HighGuid as u16,
        ObjectType::Unit => 0x70,
        ObjectType::Player => 0x70,
        ObjectType::GameObject => 0x0350,
        ObjectType::DynamicObject => 0x0150,
        ObjectType::Corpse => 0x0150,
        _ => 0,
    };

    if player.get_guid() == object.get_guid() {
        flags |= ObjectUpdateFlags::UpdateSelf as u16;
        update_type = ObjectUpdateType::CreateYourself as u8;
    }

    writer.write_u8(update_type)?;
    writer.write_guid_compressed(&object.get_guid())?;

    writer.write_u8(object.get_type() as u8)?;

    build_movement_update(&mut writer, flags, object)?;

    let mut update_mask = UpdateMask::new(object.get_num_value_fields());
    object.set_mask_for_create_bits(&mut &mut update_mask)?;
    update_mask.set_bit(ObjectFields::HighGuid as usize, true)?; //override if it wasn't detected

    build_values_update(&mut writer, object, &update_mask)?;
    block_count += 1;
    Ok((block_count, writer.into_inner()))
}

pub fn build_values_update_block(object: &dyn MapObjectWithValueFields) -> Result<(u32, Vec<u8>)> {
    let mut writer = Cursor::new(Vec::<u8>::new());

    writer.write_u8(ObjectUpdateType::Values as u8)?;
    writer.write_guid_compressed(object.get_guid())?;

    build_values_update(&mut writer, object, object.get_update_mask())?;

    Ok((1, writer.into_inner()))
}

pub fn build_out_of_range_update_block_for_player(player: &dyn MapObjectWithValueFields) -> Result<(u32, Vec<u8>)> {
    let out_of_range_guids = player.get_recently_removed_range_guids();
    if out_of_range_guids.len() == 0 {
        return Ok((0, vec![]));
    }

    let update_type = ObjectUpdateType::OutOfRangeObjects as u8;
    let mut writer = Cursor::new(Vec::<u8>::new());

    let block_count = 1;
    let num_out_of_range_guids = out_of_range_guids.len() as u32;

    writer.write_u8(update_type)?;
    writer.write_u32::<LittleEndian>(num_out_of_range_guids)?;
    for guid in out_of_range_guids {
        writer.write_guid_compressed(guid)?;
    }

    Ok((block_count, writer.into_inner()))
}

pub fn build_movement_update_block(object: &dyn MapObjectWithValueFields) -> Result<(u32, Vec<u8>)> {
    let outputbuf = Vec::<u8>::new();
    let update_type = ObjectUpdateType::Movement as u8;
    let mut writer = Cursor::new(outputbuf);

    writer.write_u8(update_type)?;
    writer.write_guid_compressed(object.get_guid())?;
    build_movement_update(&mut writer, 0x70, object)?;
    Ok((1, writer.into_inner()))
}

fn build_movement_update(writer: &mut Cursor<Vec<u8>>, flags: u16, object: &dyn MapObjectWithValueFields) -> Result<()> {
    writer.write_u16::<LittleEndian>(flags)?;

    //Only implemented for living things for now
    if flags & (ObjectUpdateFlags::Living as u16) > 0 {
        let movement_info = object.get_movement_info();
        writer.write_movement_info(movement_info)?;

        writer.write_f32::<LittleEndian>(1.0f32)?; //Walk speed
        writer.write_f32::<LittleEndian>(8.0f32)?; //Run speed
        writer.write_f32::<LittleEndian>(4.5f32)?; //backwards walk speed
        writer.write_f32::<LittleEndian>(1.00f32)?; //Swim speed
        writer.write_f32::<LittleEndian>(1.00f32)?; //back Swim speed
        writer.write_f32::<LittleEndian>(0.0f32)?; //Fly Speed, fly disabled for now so set to 0
        writer.write_f32::<LittleEndian>(0.0f32)?; //Backwards fly Speed, fly disabled for now
        writer.write_f32::<LittleEndian>(3.14f32)?; //turn speed
        writer.write_f32::<LittleEndian>(7.0)?; //pitch speed
    }
    //TODO else not living stuff

    if flags & (ObjectUpdateFlags::LowGuid as u16) > 0 {
        writer.write_u32::<LittleEndian>(object.get_guid().get_low_part())?;
    }
    if flags & (ObjectUpdateFlags::HighGuid as u16) > 0 {
        //writer.write_u32::<LittleEndian>(object.get_guid().get_high_part())?;
        if flags & (ObjectUpdateFlags::UpdateSelf as u16) > 0 {
            writer.write_u32::<LittleEndian>(0x2F)?;
        } else {
            writer.write_u32::<LittleEndian>(0x08)?;
        }
    }

    //todo: flag has target
    //todo: flag is vehicle
    //todo: flag rotation
    //todo: flag transport

    Ok(())
}

fn build_values_update(writer: &mut Cursor<Vec<u8>>, object: &dyn MapObjectWithValueFields, update_mask: &UpdateMask) -> Result<()> {
    let num_values = object.get_num_value_fields();
    assert_eq!(num_values, update_mask.get_num_fields());

    writer.write_u8(update_mask.get_num_blocks() as u8)?;
    for block in update_mask.get_blocks() {
        writer.write_u32::<LittleEndian>(*block)?;
    }

    for i in 0..num_values {
        if update_mask.get_bit(i)? {
            writer.write_u32::<LittleEndian>(object.get_field_u32(i)?)?;
        }
    }

    Ok(())
}
