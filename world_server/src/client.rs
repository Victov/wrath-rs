use super::character::*;
use crate::character::character_manager::CharacterManager;
use crate::connection::events::ServerEvent;
use crate::data::DataStorage;
use crate::handlers::login_handler::LogoutState;
use crate::prelude::*;
use crate::world::World;
use std::net::SocketAddr;
use std::sync::Arc;
use wow_world_messages::Guid;

#[derive(Clone, PartialEq, Eq)]
pub enum ClientState {
    PreLogin,
    CharacterSelection,
    DisconnectPendingCleanup,
    Disconnected,
}

pub struct ClientData {
    pub client_state: ClientState,
    pub account_id: u32,
    pub active_character: Option<Guid>,
}

pub struct Client {
    pub id: SocketAddr,

    pub connection_sender: flume::Sender<ServerEvent>,

    pub data: ClientData,
}

impl Client {
    pub fn new(id: SocketAddr, account_id: u32, connection_sender: flume::Sender<ServerEvent>) -> Self {
        Self {
            id,
            connection_sender,

            data: ClientData {
                client_state: ClientState::CharacterSelection,
                account_id,
                active_character: None,
            },
        }
    }

    pub async fn tick(&mut self, delta_time: f32, character_manager: &mut CharacterManager, world: &mut World) -> Result<()> {
        let mut should_return_to_character_select: bool = false;
        if let Some(guid) = self.data.active_character {
            let character = character_manager.get_character_mut(guid)?;
            character.tick(delta_time, world).await?;

            should_return_to_character_select = character.logout_state == LogoutState::ReturnToCharSelect;
        }

        if should_return_to_character_select {
            let data = &mut self.data;
            data.active_character = None;
            data.client_state = ClientState::CharacterSelection;
        }
        Ok(())
    }

    pub fn get_active_character(&self) -> Guid {
        self.data.active_character.unwrap()
    }

    pub fn disconnected_post_cleanup(&mut self) -> Result<()> {
        //Cleanup time has passed. Now this client is really really disconnected and
        //will be fully removed from memory
        let data = &mut self.data;
        data.client_state = ClientState::Disconnected;
        data.active_character = None;
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.data.client_state != ClientState::PreLogin
    }

    pub async fn load_and_set_active_character(
        &mut self,
        character_manager: &mut CharacterManager,
        data_storage: &Arc<DataStorage>,
        world: &World,
        character_guid: Guid,
    ) -> Result<()> {
        // TODO: send a message to the character manager?
        let character = Character::load(self.connection_sender.clone(), character_guid, world, data_storage).await?;
        character_manager.add_character(character);
        self.data.active_character.replace(character_guid);
        Ok(())
    }

    pub fn set_active_character(&mut self, character_guid: Guid) {
        self.data.active_character.replace(character_guid);
    }

    pub async fn login_active_character(&self, world: &mut World, character_manager: &mut CharacterManager) -> Result<()> {
        let data = &self.data;
        let character = character_manager.get_character_mut(data.active_character.unwrap())?;
        character.send_packets_before_add_to_map().await?;

        world
            .get_instance_manager_mut()
            .get_or_create_map(character, character.map)
            .await?
            .push_character(character);

        character.send_packets_after_add_to_map(world.get_realm_database()).await?;

        Ok(())
    }
}
