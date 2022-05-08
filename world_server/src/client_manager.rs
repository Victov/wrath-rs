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
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use wrath_auth_db::AuthDatabase;

pub struct ClientManager {
    pub auth_db: Arc<AuthDatabase>,
    pub data_storage: Arc<DataStorage>,
    pub realm_seed: u32,
    clients: RwLock<HashMap<u64, Arc<Client>>>,
}

impl ClientManager {
    pub fn new(auth_db: Arc<AuthDatabase>, data_storage: Arc<DataStorage>) -> Self {
        Self {
            auth_db,
            data_storage,
            realm_seed: rand::thread_rng().next_u32(),
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

            let realm_seed = self.realm_seed;

            client.send_auth_challenge(realm_seed).await.unwrap_or_else(|e| {
                error!("Error while sending auth challenge: {:?}", e);
            });

            let packet_channel_for_client = packet_handle_sender.clone();
            task::spawn(async move {
                handle_incoming_packets(client.clone(), read_socket_wrapped, packet_channel_for_client)
                    .await
                    .unwrap_or_else(|e| {
                        warn!("Error while handling packet {:?}", e);
                    });

                //Client stopped handling packets. Probably disconnected. Remove from client list?
                client
                    .disconnect()
                    .await
                    .unwrap_or_else(|e| warn!("Something went wrong while disconnecting client: {:?}", e));
            });
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

async fn handle_incoming_packets(client: Arc<Client>, socket: Arc<RwLock<TcpStream>>, packet_channel: Sender<PacketToHandle>) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    let mut read_length;
    loop {
        {
            let mut read_socket = socket.write().await;
            read_length = read_socket.read(&mut buf).await?;
            if read_length == 0 {
                //Disconnected, break the loop and exit the function
                break;
            }
        }
        let mut ptr = 0;
        while ptr < read_length {
            let header = super::packet::read_header(&buf, ptr, &client).await?;
            let payload_length = header.length as usize;
            let shrunk_buf = buf.iter().skip(ptr + 6).take(payload_length).copied().collect();
            packet_channel.send(PacketToHandle {
                client_id: client.id,
                header,
                payload: shrunk_buf,
            })?;
            ptr += 6 + payload_length;
        }
    }
    Ok(())
}
