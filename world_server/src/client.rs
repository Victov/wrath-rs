use anyhow::{Result};
use async_std::net::{TcpStream};
use async_std::sync::{RwLock, Mutex};
use num_bigint::{RandBigInt};
use rand::RngCore;
use std::sync::{Arc};
use super::packet::*;
use super::opcodes::Opcodes;
use super::wowcrypto::*;

#[derive(PartialEq)]
pub enum ClientState
{
    PreLogin,
    CharacterSelection,
}

pub struct Client
{
    pub read_socket: Arc<RwLock<TcpStream>>, 
    pub write_socket: Arc<Mutex<TcpStream>>,
    pub client_state : ClientState,
    pub id: u64,
    pub crypto: RwLock<ClientCrypto>,
    pub account_id: Option<u32>,
}

impl Client
{
    pub fn new(read_socket : Arc<RwLock<TcpStream>>, write_socket: Arc<Mutex<TcpStream>>) -> Self
    {
        Self
        {
            read_socket,
            write_socket, 
            client_state : ClientState::PreLogin,
            id: rand::thread_rng().next_u64(),
            crypto: RwLock::new(ClientCrypto::new()),
            account_id : None,
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

        send_packet(self, header, &writer).await?;
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool
    {
        self.account_id.is_some() && self.client_state != ClientState::PreLogin
    }
}
