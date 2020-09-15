use super::opcodes::{Opcodes};
use super::client::Client;
use anyhow::{anyhow, Result};
use async_std::prelude::*;
use std::convert::TryFrom;
use std::io::Cursor;
use podio::{LittleEndian, BigEndian, WritePodExt};

pub struct ServerPacketHeader
{
    opcode: u16,
    _length: u16,
}

pub struct ClientPacketHeader
{
    pub opcode: u32,
    pub length: u16,
}

impl ClientPacketHeader
{
    pub fn get_cmd(&self) -> Result<Opcodes>
    {
        Ok(Opcodes::try_from(self.opcode)?)
    }
}

impl ServerPacketHeader
{
    pub fn get_cmd(&self) -> Result<Opcodes>
    {
        Ok(Opcodes::try_from(self.opcode as u32)?)
    }
}

pub fn create_packet(opcode: Opcodes, allocate_size:usize) -> (ServerPacketHeader, Cursor<Vec<u8>>)
{
    let header = ServerPacketHeader{
        opcode: opcode as u16,
        _length: 0
    };

    let buf : Vec<u8> = Vec::with_capacity(allocate_size);
    let writer = std::io::Cursor::new(buf);

    (header, writer)
}

pub async fn send_packet(client: &Client, header: ServerPacketHeader, payload: &Cursor<Vec<u8>>) -> Result<()>
{
    use std::io::Write;

    let payload_length = payload.get_ref().len();

    let mut header_buffer = Vec::<u8>::new();
    let mut header_writer = std::io::Cursor::new(&mut header_buffer);
    header_writer.write_u16::<BigEndian>(payload_length as u16 + 2u16)?;
    header_writer.write_u16::<LittleEndian>(header.opcode)?;

    if client.crypto.read().await.is_initialized()
    {
        client.crypto.write().await.encrypt(&mut header_buffer)?;
    }

    let final_buf = vec![0u8; payload_length + 4];
    let mut final_writer = std::io::Cursor::new(final_buf);
    final_writer.write(&header_buffer)?;
    final_writer.write(&payload.get_ref())?;

    {
        let mut write_socket = client.socket.write().await;
        let written_len = write_socket.write(&final_writer.into_inner()).await?;
        write_socket.flush().await?;
        if written_len != payload_length + 4
        {
            return Err(anyhow!("Wrong written length while sending packet"));
        }
    }

    Ok(())
}

pub async fn read_header(bytes: &mut Vec<u8>, client: &Client) -> Result<ClientPacketHeader>
{
    use podio::{ReadPodExt};

    let mut header = bytes.iter().take(6).map(|a| *a).collect::<Vec<u8>>();

    if client.crypto.read().await.is_initialized()
    {
        client.crypto.write().await.decrypt(&mut header)?;
    }

    let mut reader = std::io::Cursor::new(header);
    let packet_len = reader.read_u16::<BigEndian>()?;
    let opcode_u32 = reader.read_u32::<LittleEndian>()?;

    let header = ClientPacketHeader
    {
        opcode: opcode_u32,
        length: packet_len
    };

    Ok(header)
}
