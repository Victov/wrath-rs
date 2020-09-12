use anyhow::{Result};
use async_std::prelude::*;
use async_std::net::{TcpStream};
use async_std::sync::{RwLock};
use num_bigint::RandBigInt;
use rand::RngCore;
use std::sync::mpsc::{Sender};
use std::sync::{Arc};
use super::packet_handler::{PacketToHandle};
use super::packet::*;
use super::opcodes::Opcodes;

#[derive(PartialEq)]
enum ClientState
{
    PreLogin,
    InWorld
}

pub struct Client
{
    socket: Arc<RwLock<TcpStream>>, 
    client_state : ClientState,
    id: u64,
}

impl Client
{
    pub fn new(socket : Arc<RwLock<TcpStream>>) -> Self
    {
        Self
        {
            socket: socket,
            client_state : ClientState::PreLogin,
            id: rand::thread_rng().next_u64(),
        }
    }

    pub async fn send_auth_challenge(&self, realm_seed: u32) -> Result<()>
    {
        use podio::{LittleEndian, WritePodExt};
        use std::io::Write;
        
        let (header, mut writer) = create_packet(Opcodes::SMSG_AUTH_CHALLENGE, 44);
        writer.write_u32::<LittleEndian>(1)?;
        writer.write_u32::<LittleEndian>(realm_seed)?;
        let seed1 = rand::thread_rng().gen_biguint(32*8);
        writer.write(&seed1.to_bytes_le())?;

        send_packet(self.socket.clone(), &writer, header).await?;
        Ok(())
    }

    pub async fn handle_incoming_packets(&self, packet_channel: Sender<PacketToHandle>) -> Result<()>
    {
        let mut buf = vec![0u8; 1024];
        let mut read_length;
        loop
        {
            {
                let mut write_socket = self.socket.write().await;
                read_length = write_socket.read(&mut buf).await?;
                if read_length == 0
                {
                    println!("disconnect");
                    write_socket.shutdown(async_std::net::Shutdown::Both)?;
                    break;
                }
            }
            let header = read_header(&buf, read_length, false)?;

            println!("Opcode = {:?}, length = {}", header.get_cmd(), header.length);
            packet_channel.send(PacketToHandle { client_id: self.id, header })?;

            /*if header.get_cmd()? == Opcodes::CMSG_AUTH_SESSION
            {
                self.handle_auth_session(&buf).await?
            }*/
        }

        Ok(())
    }

    /*
    pub async fn handle_auth_session(&mut self, packet: &[u8]) -> Result<()>
    {
        use podio::{ReadPodExt, LittleEndian};
        use std::io::{BufRead, Seek, SeekFrom};

        if self.client_state != ClientState::PreLogin
        {
            return Err(anyhow!("Client sent auth session but was already logged in"));
        }
        
        let mut reader = std::io::Cursor::new(packet);
        reader.seek(std::io::SeekFrom::Start(6))?; //skip header
        let build_number = reader.read_u32::<LittleEndian>()?;
        let _unknown1  = reader.read_u32::<LittleEndian>()?;
        let mut name = Vec::new();
        reader.read_until(0, &mut name)?;
        let name = String::from_utf8(name)?;
        let _unknown2 = reader.read_u32::<LittleEndian>()?;
        let _client_seed = reader.read_u32::<LittleEndian>()?;
        reader.seek(SeekFrom::Current(20))?; //skip unknown bytes
        let client_digest = reader.read_exact(20)?;
        let compressed_addon_data_length = reader.read_u32::<LittleEndian>()?;
        let _compressed_addon_data = reader.read_exact(compressed_addon_data_length as usize)?;

        println!("user {} connecting from buildnumer {}", name, build_number);
        println!("digest: {:?}", client_digest);
        println!("also {} bytes of addon data", compressed_addon_data_length);


        Ok(())
    }*/
}
