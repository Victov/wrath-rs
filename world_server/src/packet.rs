use super::opcodes::{Opcodes};
use anyhow::{anyhow, Result};
use async_std::net::TcpStream;
use async_std::sync::RwLock;
use async_std::prelude::*;
use std::convert::TryFrom;
use std::sync::Arc;

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

pub fn create_packet(opcode: Opcodes, allocate_size:usize) -> (ServerPacketHeader, std::io::Cursor<Vec<u8>>)
{
    let header = ServerPacketHeader{
        opcode: opcode as u16,
        _length: 0
    };

    let buf : Vec<u8> = Vec::with_capacity(allocate_size);
    let writer = std::io::Cursor::new(buf);

    (header, writer)
}

pub async fn send_packet(socket: Arc<RwLock<TcpStream>>, payload: &std::io::Cursor<Vec<u8>>, header: ServerPacketHeader) -> Result<()>
{
    use podio::{LittleEndian, BigEndian, WritePodExt};
    use std::io::Write;

    let payload_length = payload.get_ref().len();
    let final_buf = vec![0u8; payload_length + 4];
    let mut final_writer = std::io::Cursor::new(final_buf);
    final_writer.write_u16::<BigEndian>(payload_length as u16)?;
    final_writer.write_u16::<LittleEndian>(header.opcode)?;
    final_writer.write(&payload.get_ref())?;

    {
        let mut write_socket = socket.write().await;
        let written_len = write_socket.write(&final_writer.into_inner()).await?;
        write_socket.flush().await?;

        if written_len != payload_length + 4
        {
            return Err(anyhow!("Wrong written length while sending packet"));
        }
    }

    Ok(())
}

pub fn read_header(bytes: &Vec<u8>, _packet_length: usize, is_encrypted: bool) -> Result<ClientPacketHeader>
{
    use podio::{LittleEndian, BigEndian, ReadPodExt};

    if is_encrypted
    {
        return Err(anyhow!("Path not implemented"));
    }

    let mut reader = std::io::Cursor::new(bytes);
    let packet_len = reader.read_u16::<BigEndian>()?;
    let opcode_u32 = reader.read_u32::<LittleEndian>()?;

    let header = ClientPacketHeader
    {
        opcode: opcode_u32,
        length: packet_len
    };

    Ok(header)
}
