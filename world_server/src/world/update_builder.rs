//see https://github.com/arcemu/arcemu/blob/master/src/world/Game/Entities/Update/UpdateBuilder.cpp
//for reference

use crate::guid::WriteGuid;
use crate::prelude::*;
use podio::{LittleEndian, WritePodExt};
use std::io::Cursor;

use super::super::constants::updates::*;
use super::prelude::{HasValueFields, MapObject, ObjectFields, UpdateMask};

pub trait ReceiveUpdates {
    fn push_creation_data(&mut self, data: &mut Vec<u8>, block_count: u32);
    fn get_creation_data(&self) -> (u32, &Vec<u8>);
    fn clear_creation_data(&mut self);
}

pub fn build_create_update_block_for_player(player: &impl MapObject, object: &(impl MapObject + HasValueFields)) -> Result<(u32, Vec<u8>)> {
    let flags2: u32 = 0;
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
    //We should be writing the object guid, however since we still have a hardcoded
    //GUID in the build_values_update they need to match. So until we have flexible
    //values update that's built up from actual values, leave the temp hardcoded guid here
    writer.write_guid_compressed(&object.get_guid())?;

    //let guid = Guid::new(0x00010203, 0, guid::HighGuid::Player);
    //writer.write_guid_compressed(&guid)?;

    writer.write_u8(object.get_type() as u8)?;

    build_movement_update(&mut writer, flags, flags2, player, object)?;

    let mut update_mask = UpdateMask::new(object.get_num_value_fields());
    object.set_mask_for_create_bits(&mut &mut update_mask)?;
    update_mask.set_bit(ObjectFields::HighGuid as usize, true)?; //override if it wasn't detected

    build_values_update(&mut writer, player, object, &update_mask)?;
    block_count += 1;
    Ok((block_count, writer.into_inner()))
}

fn build_movement_update(writer: &mut Cursor<Vec<u8>>, flags: u16, flags2: u32, _player: &impl MapObject, object: &impl MapObject) -> Result<()> {
    writer.write_u16::<LittleEndian>(flags)?;

    //Only implemented for living things for now
    if flags & (ObjectUpdateFlags::Living as u16) > 0 {
        writer.write_u32::<LittleEndian>(flags2)?;
        writer.write_u16::<LittleEndian>(0)?; //extra move flags (vehicles stuff)
        writer.write_u32::<LittleEndian>(0)?; //time stamp milliseconds?

        let position = object.get_position();
        writer.write_f32::<LittleEndian>(position.x)?;
        writer.write_f32::<LittleEndian>(position.y)?;
        writer.write_f32::<LittleEndian>(position.z)?;
        writer.write_f32::<LittleEndian>(position.o)?;

        writer.write_u32::<LittleEndian>(0)?; // fall time

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
        writer.write_u32::<LittleEndian>(object.get_guid().get_high_part())?;
    }

    //todo: flag has target
    //todo: flag is vehicle
    //todo: flag rotation
    //todo: flag transport

    Ok(())
}

fn build_values_update(
    writer: &mut Cursor<Vec<u8>>,
    _player: &impl MapObject,
    object: &(impl MapObject + HasValueFields),
    update_mask: &UpdateMask,
) -> Result<()> {
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
