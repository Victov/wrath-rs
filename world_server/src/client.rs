use anyhow::{Result};
use async_std::net::{TcpStream};
use async_std::sync::{RwLock};
use num_bigint::RandBigInt;
use rand::RngCore;
use std::sync::{Arc};
use super::packet::*;
use super::opcodes::Opcodes;
use super::wowcrypto::*;

#[derive(PartialEq)]
pub enum ClientState
{
    PreLogin,
}

pub struct Client
{
    pub socket: Arc<RwLock<TcpStream>>, 
    pub client_state : ClientState,
    pub id: u64,
    crypto: ClientCrypto
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
            crypto: ClientCrypto::new(),
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

    pub fn init_crypto(&mut self, sess_key: &[u8]) -> Result<()>
    {
        self.crypto.initialize(sess_key)?;
        Ok(())
    }
}
