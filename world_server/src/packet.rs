use super::opcodes::{Opcodes};
use anyhow::{anyhow, Result};
use async_std::net::TcpStream;
use async_std::prelude::*;

pub struct ServerPacketHeader
{
    opcode: u16,
    _length: u16,
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

pub async fn send_packet(socket: &mut TcpStream, payload: &std::io::Cursor<Vec<u8>>, header: ServerPacketHeader) -> Result<()>
{
    use podio::{LittleEndian, BigEndian, WritePodExt};
    use std::io::Write;

    let payload_length = payload.get_ref().len();
    let final_buf = vec![0u8; payload_length + 4];
    let mut final_writer = std::io::Cursor::new(final_buf);
    final_writer.write_u16::<BigEndian>(payload_length as u16)?;
    final_writer.write_u16::<LittleEndian>(header.opcode)?;
    final_writer.write(&payload.get_ref())?;

    let written_len = socket.write(&final_writer.into_inner()).await?;
    socket.flush().await?;

    if written_len != payload_length + 4
    {
        return Err(anyhow!("Wrong written length while sending packet"));
    }

    Ok(())
}
