use super::client::*;
use super::packet_handler::PacketToHandle;
use crate::data::DataStorage;
use crate::prelude::*;
use crate::world::World;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::stream::StreamExt;
use async_std::sync::{Mutex, RwLock};
use async_std::task;
use rand::{thread_rng, RngCore};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use wow_srp::wrath_header::ServerCrypto;
use wow_world_messages::wrath::opcodes::ClientOpcodeMessage;
use wrath_auth_db::AuthDatabase;

pub struct ClientManager {
    pub auth_db: Arc<AuthDatabase>,
    pub data_storage: Arc<DataStorage>,
    clients: RwLock<HashMap<u64, Arc<Client>>>,
}

impl ClientManager {
    pub fn new(auth_db: Arc<AuthDatabase>, data_storage: Arc<DataStorage>) -> Self {
        Self {
            auth_db,
            data_storage,
            clients: RwLock::new(HashMap::new()),
        }
    }

    pub async fn tick(&self, delta_time: f32, world: Arc<World>) -> Result<()> {
        self.cleanup_disconnected_clients(world.clone()).await?;
        let clients = self.clients.read().await;
        for (_, client) in clients.iter() {
            client.tick(delta_time, world.clone()).await?;
        }

        Ok(())
    }

    async fn cleanup_disconnected_clients(&self, world: Arc<World>) -> Result<()> {
        let to_remove = {
            let mut result = vec![];
            let clients = self.clients.read().await;
            for (id, client) in clients.iter() {
                //Cleanup is two-staged. Sockets are already closed here, but we take this frame to
                //be able to remove them from the world and all that cleanup
                let client_state = {
                    let data = client.data.read().await;
                    data.client_state.clone()
                };
                if client_state == ClientState::DisconnectPendingCleanup {
                    world.get_instance_manager().handle_client_disconnected(client).await?;
                    //insert more cleanup actions here
                    client.disconnected_post_cleanup().await?;
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

        let mut write_clients = self.clients.write().await;
        write_clients.retain(|id, _| !to_remove.contains(id));
        info!("Cleaned up {} clients, {} clients left online", to_remove.len(), write_clients.len());

        Ok(())
    }

    pub async fn accept_realm_connections(&self, packet_handle_sender: Sender<PacketToHandle>) -> Result<()> {
        let realm_id: i32 = std::env::var("REALM_ID")?.parse()?;
        let bind_ip = self.auth_db.get_realm_bind_ip(realm_id).await?;
        let tcp_listener = TcpListener::bind(bind_ip).await?;
        let mut incoming_connections = tcp_listener.incoming();

        while let Some(tcp_stream) = incoming_connections.next().await {
            let read_stream = tcp_stream?;
            let write_stream = read_stream.clone();
            let read_socket_wrapped = Arc::new(RwLock::new(read_stream));
            let write_socket_wrapped = Arc::new(Mutex::new(write_stream));
            let client_id = thread_rng().next_u64();
            let client = Arc::new(Client::new(client_id, read_socket_wrapped.clone(), write_socket_wrapped));

            {
                let mut hashmap = self.clients.write().await;
                hashmap.insert(client_id, client.clone());
            }
            {
                //Have to make local copies of all these things to avoid `self` references in the
                //asyncmove block.
                let client = client.clone();
                let packet_handle_sender = packet_handle_sender.clone();
                let auth_db = self.auth_db.clone();
                task::spawn(async move {
                    let p = packet_handle_sender.clone();
                    let a_db = auth_db.clone();
                    client.authenticate_and_start_receiving_data(p, a_db).await;
                });
            }
        }

        Ok(())
    }

    pub async fn get_authenticated_client(&self, id: u64) -> Result<Arc<Client>> {
        let client = self.get_client(id).await?;
        if !client.is_authenticated().await {
            bail!("Character isn't authenticated");
        }
        Ok(client)
    }

    pub async fn get_client(&self, id: u64) -> Result<Arc<Client>> {
        let hashmap = self.clients.read().await;
        let clientlock = hashmap.get(&id).ok_or_else(|| anyhow!("Failed to get client for client id: {}", id))?;
        Ok(clientlock.clone())
    }
}
