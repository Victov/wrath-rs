use super::character::*;
use super::client_manager::ClientManager;
use super::guid::*;
use super::opcodes::Opcodes;
use super::packet::*;
use super::wowcrypto::*;
use crate::prelude::*;
use crate::world::prelude::ReceiveUpdates;
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
    pub active_character: Option<Arc<RwLock<Character>>>,
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
            active_character: None,
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

    pub async fn load_and_set_active_character(&mut self, client_manager: &ClientManager, character_guid: Guid) -> Result<()> {
        let weakself = Arc::downgrade(&client_manager.get_client(self.id).await?.clone());
        let mut character = Character::new(weakself, character_guid);
        character.load_from_database(&client_manager.realm_db).await?;
        self.active_character = Some(Arc::new(RwLock::new(character)));
        Ok(())
    }

    pub async fn login_active_character(&self, client_manager: &ClientManager) -> Result<()> {
        let character_lock = self.active_character.as_ref().unwrap();
        let character_instance_id;

        //Operations that can happen within a read lock
        {
            let character = character_lock.read().await;
            character_instance_id = character.instance_id;
            character.perform_login(client_manager).await?;
        }

        //This one must have no locks because it needs a write lock
        client_manager
            .world
            .get_instance_manager()
            .get_map_for_instance(character_instance_id)
            .await
            .push_object(Arc::downgrade(&character_lock))
            .await?;

        //We need write for this
        {
            let mut character = character_lock.write().await;
            let (num, buf) = (*character).get_creation_data();
            handlers::send_update_packet(&character, num, &buf).await?;
            character.clear_creation_data();
        }

        Ok(())
    }
}
