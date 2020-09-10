use anyhow::Result;
use async_std::prelude::*;
use async_std::net::{TcpStream};
use num_bigint::RandBigInt;
use super::packet::*;

pub struct Client
{
    socket: TcpStream, 
}

impl Client
{
    pub fn new(socket : TcpStream) -> Self
    {
        Self
        {
            socket: socket,
        }
    }

    pub async fn send_auth_challenge(&mut self, realm_seed: u32) -> Result<()>
    {
        use std::io::Write;
        use podio::{LittleEndian, WritePodExt};
        use super::opcodes::Opcodes;
        
        let (header, mut writer) = create_packet(Opcodes::SMSG_AUTH_CHALLENGE, 44);
        writer.write_u32::<LittleEndian>(1)?;
        writer.write_u32::<LittleEndian>(realm_seed)?;
        let seed1 = rand::thread_rng().gen_biguint(32*8);
        writer.write(&seed1.to_bytes_le())?;

        send_packet(&mut self.socket, &writer, header).await?;
        Ok(())
    }

    pub async fn handle_incoming_packets(&mut self) -> Result<()>
    {
        let mut buf = [0u8; 1024];
        loop
        {
            let length = self.socket.read(&mut buf).await?;
            if length == 0
            {
                println!("disconnect");
                self.socket.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
            println!("got sometihng {:?}", &buf[0..32]);
        }

        Ok(())
    }
}
