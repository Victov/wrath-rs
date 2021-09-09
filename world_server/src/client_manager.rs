use super::client::*;
use super::packet_handler::PacketToHandle;
use crate::world::World;
use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::stream::StreamExt;
use async_std::sync::{Mutex, RwLock};
use async_std::task;
use rand::RngCore;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use wrath_auth_db::AuthDatabase;
use wrath_realm_db::RealmDatabase;

pub struct ClientManager {
    pub auth_db: Arc<AuthDatabase>,
    pub realm_db: Arc<RealmDatabase>,
    pub realm_seed: u32,
    clients: RwLock<HashMap<u64, Arc<RwLock<Client>>>>,
    pub world: Arc<World>,
}

impl ClientManager {
    pub fn new(auth_db: Arc<AuthDatabase>, realm_db: Arc<RealmDatabase>, world: Arc<World>) -> Self {
        Self {
            auth_db,
            realm_db,
            realm_seed: rand::thread_rng().next_u32(),
            clients: RwLock::new(HashMap::new()),
            world,
        }
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
            let client_lock = Arc::new(RwLock::new(Client::new(read_socket_wrapped.clone(), write_socket_wrapped)));
            let client_id = client_lock.read().await.id;

            {
                let mut hashmap = self.clients.write().await;
                hashmap.insert(client_id, client_lock.clone());
            }

            let realm_seed = self.realm_seed;
            let packet_channel_for_client = packet_handle_sender.clone();

            client_lock.read().await.send_auth_challenge(realm_seed).await.unwrap_or_else(|e| {
                println!("Error while sending auth challenge: {:?}", e);
                return;
            });

            task::spawn(async move {
                handle_incoming_packets(client_lock, read_socket_wrapped, packet_channel_for_client)
                    .await
                    .unwrap_or_else(|e| {
                        println!("Error while handling packet {:?}", e);
                    });
                //Client stopped handling packets. Probably disconnected. Remove from client list?
            });
        }

        Ok(())
    }

    pub async fn get_client(&self, id: u64) -> Result<Arc<RwLock<Client>>> {
        let hashmap = self.clients.read().await;
        let clientlock = hashmap
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("Failed to get client for client id: {}", id))?;
        Ok(clientlock.clone())
    }
}

async fn handle_incoming_packets(
    client_lock: Arc<RwLock<Client>>,
    socket: Arc<RwLock<TcpStream>>,
    packet_channel: Sender<PacketToHandle>,
) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    let mut read_length;
    loop {
        {
            let mut read_socket = socket.write().await;
            read_length = read_socket.read(&mut buf).await?;
            if read_length == 0 {
                println!("disconnect");
                read_socket.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
        }
        let mut ptr = 0;
        while ptr < read_length {
            let client = client_lock.read().await;
            let header = super::packet::read_header(&buf, ptr, &client).await?;
            let payload_length = header.length as usize;
            let shrunk_buf = buf.iter().skip(ptr + 6).take(payload_length).map(|a| *a).collect();
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
