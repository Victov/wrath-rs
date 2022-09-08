use super::client::Client;
use super::opcodes::Opcodes;
use crate::{character::*, prelude::*, world::prelude::*};
use async_std::prelude::*;
use podio::{BigEndian, LittleEndian, WritePodExt};
use std::convert::TryFrom;
use std::io::Cursor;

pub struct ServerPacketHeader {
    opcode: u16,
    _length: u16,
}

impl ServerPacketHeader {
    pub fn get_cmd(&self) -> Result<Opcodes> {
        Ok(Opcodes::try_from(self.opcode as u32)?)
    }
}

pub fn create_packet(opcode: Opcodes, allocate_size: usize) -> (ServerPacketHeader, Cursor<Vec<u8>>) {
    let header = ServerPacketHeader {
        opcode: opcode as u16,
        _length: 0,
    };

    let buf: Vec<u8> = Vec::with_capacity(allocate_size);
    let writer = std::io::Cursor::new(buf);

    (header, writer)
}

pub async fn send_packet_to_all_in_range(
    character: &Character,
    include_self: bool,
    world: &World,
    header: &ServerPacketHeader,
    payload: &Cursor<Vec<u8>>,
) -> Result<()> {
    if let Some(map) = world.get_instance_manager().try_get_map_for_character(character).await {
        let in_range_guids = character.as_world_object().unwrap().get_in_range_guids();
        for guid in in_range_guids {
            let object_lock = map
                .try_get_object(guid)
                .await
                .ok_or_else(|| anyhow!("GUID is in range, but not a valid object"))?
                .upgrade()
                .ok_or_else(|| anyhow!("object was on the map, but is no longer valid to send packets to"))?;
            let read_obj = object_lock.read().await;
            if let Some(in_range_character) = read_obj.as_character() {
                send_packet_to_character(in_range_character, header, payload).await?;
            }
        }
        if include_self {
            send_packet_to_character(character, header, payload).await?;
        }
    } else {
        warn!("Trying to send packet to all in range, but this character is not on a map");
    }

    Ok(())
}

pub async fn send_packet_to_character(character: &Character, header: &ServerPacketHeader, payload: &Cursor<Vec<u8>>) -> Result<()> {
    let client = character
        .client
        .upgrade()
        .ok_or_else(|| anyhow!("failed to get associated client from character"))?;

    send_packet(&client, header, payload).await
}

pub async fn send_packet(client: &Client, header: &ServerPacketHeader, payload: &Cursor<Vec<u8>>) -> Result<()> {
    use std::io::Write;

    let payload_length = payload.get_ref().len();
    if payload_length > 0x7FFF {
        return Err(anyhow!("Sending large packet header, but we don't have support for that yet!"));
    }

    let mut header_buffer = Vec::<u8>::new();
    let mut header_writer = std::io::Cursor::new(&mut header_buffer);
    header_writer.write_u16::<BigEndian>(payload_length as u16 + 2u16)?;
    header_writer.write_u16::<LittleEndian>(header.opcode)?;

    /*
    if client.crypto.read().await.is_initialized() {
        client.crypto.write().await.encrypt(&mut header_buffer)?;
    }*/

    let final_buf = vec![0u8; payload_length + 4];
    let mut final_writer = std::io::Cursor::new(final_buf);
    final_writer.write_all(&header_buffer)?;
    final_writer.write_all(payload.get_ref())?;

    {
        let mut write_socket = client.write_socket.lock().await;
        let written_len = write_socket.write(&final_writer.into_inner()).await?;
        write_socket.flush().await?;

        if std::env::var("PRINT_OUTGOING_PACKETS")?.parse::<usize>()? == 1usize {
            info!("Outgoing: {:?}", header.get_cmd());
        }
        if written_len != payload_length + 4 {
            return Err(anyhow!("Wrong written length while sending packet"));
        }
    }

    Ok(())
}
