//see https://github.com/arcemu/arcemu/blob/master/src/world/Game/Entities/Update/UpdateBuilder.cpp
//for reference

use anyhow::Result;
use podio::{BigEndian, LittleEndian, WritePodExt};

use crate::guid::{self, Guid, WriteGuid};
use std::io::Cursor;

use super::super::constants::updates::*;
use super::prelude::MapObject;

pub trait ReceiveUpdates {
    fn push_creation_data(&mut self, data: &mut Vec<u8>, block_count: u32);
    fn get_creation_data(&self) -> (u32, &Vec<u8>);
    fn clear_creation_data(&mut self);
}

pub fn build_create_update_block_for_player(
    player: &impl MapObject,
    object: &impl MapObject,
) -> Result<(u32, Vec<u8>)> {
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
    //writer.write_guid_compressed(&object.get_guid())?; //should be this
    let guid = Guid::new(0x00010203, 0, guid::HighGuid::Player);
    writer.write_guid_compressed(&guid)?;

    writer.write_u8(object.get_type() as u8)?;

    build_movement_update(&mut writer, flags, flags2, player, object)?;
    build_values_update(&mut writer, player, object)?;
    block_count += 1;
    Ok((block_count, writer.into_inner()))
}

fn build_movement_update(
    writer: &mut Cursor<Vec<u8>>,
    flags: u16,
    flags2: u32,
    _player: &impl MapObject,
    object: &impl MapObject,
) -> Result<()> {
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
    _object: &impl MapObject,
) -> Result<()> {
    //temporarily hardcoded, to be done nicely later
    let values = &[
        0x2a, // Mask Size ((1326 + 31) / 32 = 42)
        0b00010111, 0x00, 0x80, 0x01, 0x01, 0x00, 0b11000000, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03,
        0x00, // OBJECT_FIELD_GUID Low GUID [Required]
        0x00, 0x00, 0x00, 0x00, // OBJECT_FIELD_GUID High GUID [Required]
        0x19, 0x00, 0x00, 0x00, // OBJECT_FIELD_TYPE -> unit | player | object
        0x00, 0x00, 0x80, 0x3f, // OBJECT_FIELD_SCALE_X
        0x01, 0x01, 0x01,
        0x01, // UNIT_FIELD_BYTES_0 Race(Human), Class(Warrior), Gender(Female), PowerType(Rage)
        0x3c, 0x00, 0x00, 0x00, // UNIT_FIELD_HEALTH
        0x3c, 0x00, 0x00, 0x00, // UNIT_FIELD_MAXHEALTH
        0x01, 0x00, 0x00, 0x00, // UNIT_FIELD_LEVEL
        0x01, 0x00, 0x00, 0x00, // UNIT_FIELD_FACTIONTEMPLATE [Required]
        0x0c, 0x4d, 0x00, 0x00, // UNIT_FIELD_DISPLAYID (Human Female = 19724) [Required]
        0x0c, 0x4d, 0x00, 0x00,
    ];

    {
        use std::io::Write;
        writer.write(values)?;
    }

    Ok(())
}
