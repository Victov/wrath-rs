use super::client::*;
use crate::character::character_manager::CharacterManager;
use crate::character::Character;
use crate::connection::events::ClientEvent;
use crate::data::DataStorage;
use crate::packet_handler::{PacketHandler, PacketToHandle};
use crate::prelude::*;
use crate::world::prelude::GameObject;
use crate::world::World;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use wrath_auth_db::AuthDatabase;

pub struct ClientManager {
    pub auth_db: Arc<AuthDatabase>,
    pub data_storage: Arc<DataStorage>,
    clients: HashMap<SocketAddr, Client>,

    sender: flume::Sender<ClientEvent>,
    pub receiver: flume::Receiver<ClientEvent>,
}

impl ClientManager {
    pub fn new(auth_db: Arc<AuthDatabase>, data_storage: Arc<DataStorage>) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            auth_db,
            data_storage,
            clients: HashMap::new(),
            sender,
            receiver,
        }
    }

    pub fn get_sender(&self) -> flume::Sender<ClientEvent> {
        self.sender.clone()
    }

    pub async fn tick(&mut self, delta_time: f32, character_manager: &mut CharacterManager, world: &mut World) -> Result<()> {
        self.cleanup_disconnected_clients(character_manager, world).await?;
        self.handle_connection_events(character_manager, world).await?;
        let clients = &mut self.clients;
        for (_, client) in clients.iter_mut() {
            client.tick(delta_time, character_manager, world).await?;
        }

        Ok(())
    }

    async fn handle_connection_events(&mut self, character_manager: &mut CharacterManager, world: &mut World) -> Result<()> {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                ClientEvent::Connected {
                    addr,
                    account_id,
                    connection_sender,
                } => {
                    let client = Client::new(addr, account_id, connection_sender);
                    self.clients.insert(addr, client);
                }
                ClientEvent::Disconnected { addr } => {
                    if let Some(client) = self.clients.get_mut(&addr) {
                        client.data.client_state = ClientState::DisconnectPendingCleanup;
                    } else {
                        error!("Received disconnect event for unknown client: {}", addr);
                    }
                }
                ClientEvent::Message { addr, packet } => {
                    let packet_to_handle = PacketToHandle {
                        client_id: addr,
                        payload: Box::new(packet),
                    };
                    PacketHandler::handle_packet(self, character_manager, world, packet_to_handle).await?;
                }
            }
        }

        Ok(())
    }

    pub fn remove_client(&mut self, client_id: SocketAddr) -> Option<Client> {
        self.clients.remove(&client_id)
    }

    async fn cleanup_disconnected_clients(&mut self, character_manager: &CharacterManager, world: &mut World) -> Result<()> {
        let to_remove = {
            let mut result = vec![];
            let clients = &mut self.clients;
            for (id, client) in clients.iter_mut() {
                //Cleanup is two-staged. Sockets are already closed here, but we take this frame to
                //be able to remove them from the world and all that cleanup
                let client_state = {
                    let data = &client.data;
                    data.client_state.clone()
                };
                if client_state == ClientState::DisconnectPendingCleanup {
                    world
                        .get_instance_manager_mut()
                        .handle_client_disconnected(client, character_manager)
                        .await?;
                    //insert more cleanup actions here
                    client.disconnected_post_cleanup()?;
                } else if client_state == ClientState::Disconnected {
                    //Here the client is disconnected and cleanup is done.
                    //insert id so we can clean that hashmap later
                    result.push(*id);
                }
            }
            result
        };
        if to_remove.is_empty() {
            return Ok(());
        }

        let write_clients = &mut self.clients;
        write_clients.retain(|id, _| !to_remove.contains(id));
        info!("Cleaned up {} clients, {} clients left online", to_remove.len(), write_clients.len());

        Ok(())
    }

    pub fn get_authenticated_client(&self, id: SocketAddr) -> Result<&Client> {
        let client = self.get_client(id)?;
        if !client.is_authenticated() {
            bail!("Character isn't authenticated");
        }
        Ok(client)
    }

    pub async fn get_authenticated_client_mut(&mut self, id: SocketAddr) -> Result<&mut Client> {
        let client = self.get_client_mut(id).await?;
        if !client.is_authenticated() {
            bail!("Character isn't authenticated");
        }
        Ok(client)
    }

    pub async fn get_character_from_client(&self, id: SocketAddr) -> Result<Guid> {
        let client = self.get_authenticated_client(id)?;
        Ok(client.get_active_character())
    }

    pub fn get_client(&self, id: SocketAddr) -> Result<&Client> {
        let hashmap = &self.clients;
        hashmap.get(&id).ok_or_else(|| anyhow!("Failed to get client for client id: {}", id))
    }

    pub async fn get_client_mut(&mut self, id: SocketAddr) -> Result<&mut Client> {
        let hashmap = &mut self.clients;
        hashmap.get_mut(&id).ok_or_else(|| anyhow!("Failed to get client for client id: {}", id))
    }

    //Attempts to find a client based on the character's name that they are currently playing.
    pub fn find_client_from_active_character_name(&self, character_name: &str, character_manager: &CharacterManager) -> Result<&Client> {
        let clients = &self.clients;
        for (_, client) in clients.iter() {
            if let Some(active_character) = client.data.active_character {
                let character = character_manager.get_character(active_character)?;
                if character.name.to_uppercase().trim() == character_name.to_uppercase().trim() {
                    return Ok(client);
                }
            }
        }

        Err(anyhow!("Failed to find client for character {}", character_name))
    }

    //Attempts to find a client based on the character's guid that they are currently playing.
    pub fn find_client_from_active_character_guid(&self, character_guid: Guid) -> Result<&Client> {
        let clients = &self.clients;
        for (_, client) in clients.iter() {
            if let Some(guid) = client.data.active_character {
                if guid == character_guid {
                    return Ok(client);
                }
            }
        }

        Err(anyhow!("Failed to find client for character {}", character_guid))
    }

    pub fn get_client_from_character(&self, character: &Character) -> Result<&Client> {
        self.find_client_from_active_character_guid(character.get_guid())
    }
}
