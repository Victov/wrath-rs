use anyhow::Result;
use async_std::prelude::*;
use async_std::net::{TcpStream};
use rand::RngCore;
use num_bigint::RandBigInt;

pub struct Client
{
    socket: TcpStream, 
    seed: u32
}

impl Client
{
    pub fn new(socket : TcpStream) -> Self
    {
        Self
        {
            socket: socket,
            seed: rand::thread_rng().next_u32()
        }
    }

    pub async fn send_auth_challenge(&mut self) -> Result<()>
    {
        use std::io::Write;
        use podio::{LittleEndian, WritePodExt};
        use super::opcodes::Opcodes;

        let buf = vec![0u8;26];
        let mut writer = std::io::Cursor::new(buf);
        writer.write_u16::<LittleEndian>(Opcodes::SMSG_AUTH_CHALLENGE as u16)?;
        writer.write_u16::<LittleEndian>(28)?; //size?
        writer.write_u32::<LittleEndian>(1)?;
        writer.write_u32::<LittleEndian>(self.seed)?;
        let seed1 = rand::thread_rng().gen_biguint(16*8);
        writer.write(&seed1.to_bytes_le())?;

        self.socket.write(&writer.into_inner()).await?;
        self.socket.flush().await?;

        Ok(())
    }

    pub async fn handle_incoming_packets(&mut self) -> Result<()>
    {
        let mut buf = vec![0u8;1024];
        loop
        {
            println!("loop");
            let length = self.socket.read(&mut buf).await?;
            if length == 0
            {
                println!("disconnect");
                self.socket.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
            println!("{:?}", buf);

        }

        Ok(())
    }
}
