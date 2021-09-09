use super::character::*;
use super::client_manager::ClientManager;
use super::guid::*;
use super::opcodes::Opcodes;
use super::packet::*;
use super::wowcrypto::*;
use anyhow::Result;
use async_std::net::TcpStream;
use async_std::sync::{Mutex, RwLock};
use num_bigint::RandBigInt;
use rand::RngCore;
use std::sync::Arc;

#[derive(PartialEq)]
pub enum ClientState {
    PreLogin,
    CharacterSelection,
}

pub struct Client {
    pub read_socket: Arc<RwLock<TcpStream>>,
    pub write_socket: Arc<Mutex<TcpStream>>,
    pub client_state: ClientState,
    pub id: u64,
    pub crypto: RwLock<ClientCrypto>,
    pub account_id: Option<u32>,
    pub active_character: Arc<RwLock<Option<Character>>>,
}

impl Client {
    pub fn new(read_socket: Arc<RwLock<TcpStream>>, write_socket: Arc<Mutex<TcpStream>>) -> Self {
        Self {
            read_socket,
            write_socket,
            client_state: ClientState::PreLogin,
            id: rand::thread_rng().next_u64(),
            crypto: RwLock::new(ClientCrypto::new()),
            account_id: None,
            active_character: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn send_auth_challenge(&self, realm_seed: u32) -> Result<()> {
        use podio::{LittleEndian, WritePodExt};
        use std::io::Write;

        let (header, mut writer) = create_packet(Opcodes::SMSG_AUTH_CHALLENGE, 44);
        writer.write_u32::<LittleEndian>(1)?;
        writer.write_u32::<LittleEndian>(realm_seed)?;
        let seed1 = rand::thread_rng().gen_biguint(32 * 8);
        writer.write(&seed1.to_bytes_le())?;

        send_packet(self, header, &writer).await?;
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.account_id.is_some() && self.client_state != ClientState::PreLogin
    }

    pub async fn login_character(&self, client_manager: &ClientManager, character_guid: Guid) -> Result<()> {
        //Load character and insert into our safe locks asap
        {
            let weakself = Arc::downgrade(&client_manager.get_client(self.id).await?.clone());
            let character = Character::load_from_database(weakself, &client_manager.realm_db, character_guid).await?;
            *self.active_character.write().await = Some(character);
        }
        //take the route that any other caller would take, through acquiring a lock
        let lock = self.active_character.read().await;
        let character = lock.as_ref().unwrap();
        character.perform_login(client_manager).await?;

        Ok(())
    }
}
