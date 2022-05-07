use super::character::*;
use super::client_manager::ClientManager;
use super::opcodes::Opcodes;
use super::packet::*;
use super::wowcrypto::*;
use crate::handlers::login_handler::LogoutState;
use crate::prelude::*;
use crate::world::World;
use async_std::net::TcpStream;
use async_std::sync::{Mutex, RwLock};
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub enum ClientState {
    PreLogin,
    CharacterSelection,
    DisconnectPendingCleanup,
    Disconnected,
}

pub struct ClientData {
    pub client_state: ClientState,
    pub account_id: Option<u32>,
    pub active_character: Option<Arc<RwLock<Character>>>,
}

pub struct Client {
    pub id: u64,
    pub read_socket: Arc<RwLock<TcpStream>>,
    pub write_socket: Arc<Mutex<TcpStream>>,
    pub crypto: RwLock<ClientCrypto>,
    pub data: RwLock<ClientData>,
}

impl Client {
    pub fn new(id: u64, read_socket: Arc<RwLock<TcpStream>>, write_socket: Arc<Mutex<TcpStream>>) -> Self {
        Self {
            id,
            read_socket,
            write_socket,
            crypto: RwLock::new(ClientCrypto::new()),
            data: RwLock::new(ClientData {
                client_state: ClientState::PreLogin,
                account_id: None,
                active_character: None,
            }),
        }
    }

    pub async fn tick(&self, delta_time: f32, world: Arc<World>) -> Result<()> {
        let mut should_return_to_character_select: bool = false;

        if let Some(character_lock) = &self.data.read().await.active_character {
            let mut character = character_lock.write().await;
            character.tick(delta_time, world).await?;

            should_return_to_character_select = character.logout_state == LogoutState::ReturnToCharSelect;
        }

        if should_return_to_character_select {
            let data = &mut self.data.write().await;
            data.active_character = None;
            data.client_state = ClientState::CharacterSelection;
        }
        Ok(())
    }

    pub async fn get_active_character(&self) -> Result<Arc<RwLock<Character>>> {
        let data = self.data.read().await;
        let arc = data.active_character.as_ref().ok_or_else(|| anyhow!("No active character"))?;
        Ok(arc.clone())
    }

    pub async fn disconnect(&self) -> Result<()> {
        //Kill all networking, but allow the world one frame to do cleanup
        //For example, keep around the active character, so that the instance manager can see that
        //we're disconnected, but still access the character to do world cleanup
        info!("A client disconnected");

        {
            let data = &mut self.data.write().await;
            data.client_state = ClientState::DisconnectPendingCleanup;
        }
        self.read_socket.write().await.shutdown(async_std::net::Shutdown::Both)?;
        self.write_socket.lock().await.shutdown(std::net::Shutdown::Both)?;
        //Save character to db?
        Ok(())
    }

    pub async fn disconnected_post_cleanup(&self) -> Result<()> {
        //Cleanup time has passed. Now this client is really really disconnected and
        //will be fully removed from memory
        let data = &mut self.data.write().await;
        data.client_state = ClientState::Disconnected;
        data.account_id = None;
        data.active_character = None;
        Ok(())
    }

    pub async fn send_auth_challenge(&self, realm_seed: u32) -> Result<()> {
        use num_bigint::RandBigInt;
        use podio::{LittleEndian, WritePodExt};
        use std::io::Write;

        let (header, mut writer) = create_packet(Opcodes::SMSG_AUTH_CHALLENGE, 44);
        writer.write_u32::<LittleEndian>(1)?;
        writer.write_u32::<LittleEndian>(realm_seed)?;
        let seed1 = rand::thread_rng().gen_biguint(32 * 8);
        writer.write_all(&seed1.to_bytes_le())?;

        send_packet(self, &header, &writer).await?;
        Ok(())
    }

    pub async fn is_authenticated(&self) -> bool {
        let data = self.data.read().await;
        data.account_id.is_some() && data.client_state != ClientState::PreLogin
    }

    pub async fn load_and_set_active_character(&self, client_manager: &ClientManager, character_guid: Guid) -> Result<()> {
        let weakself = Arc::downgrade(&client_manager.get_client(self.id).await?);
        let mut character = Character::new(weakself, character_guid);
        character
            .load_from_database(&client_manager.data_storage, &client_manager.world.get_realm_database())
            .await?;
        let mut data = self.data.write().await;
        data.active_character = Some(Arc::new(RwLock::new(character)));
        Ok(())
    }

    pub async fn login_active_character(&self, world: &World) -> Result<()> {
        let data = self.data.read().await;
        let character_lock = data.active_character.as_ref().unwrap();
        let character = character_lock.read().await;
        character.send_packets_before_add_to_map().await?;

        world
            .get_instance_manager()
            .get_or_create_map(&(*character), character.map)
            .await?
            .push_object(Arc::downgrade(character_lock))
            .await;

        character.send_packets_after_add_to_map(world.get_realm_database()).await?;

        Ok(())
    }
}
