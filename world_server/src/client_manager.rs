use anyhow::Result;
use async_std::task;
use async_std::net::{TcpListener, TcpStream};
use async_std::stream::{StreamExt};
use async_std::prelude::*;
use async_std::sync::RwLock;
use wrath_auth_db::{AuthDatabase};
use super::client::*;
use std::sync::Arc;
use std::sync::mpsc::{Sender};
use super::packet_handler::{PacketToHandle};
use std::collections::HashMap;
use rand::RngCore;

pub struct ClientManager
{
    auth_db : Arc<AuthDatabase>,
    realm_seed : u32,
    clients: RwLock<HashMap<u64, Arc<RwLock<Client>>>>,
}

impl ClientManager
{
    pub fn new(auth_db : Arc<AuthDatabase>) -> Self
    {
        Self
        {
            auth_db,
            realm_seed : rand::thread_rng().next_u32(),
            clients: RwLock::new(HashMap::new()),
        }
    }

    pub async fn accept_realm_connections(&self, packet_handle_sender: Sender<PacketToHandle>) -> Result<()>
    {
        let realm_id : i32 = std::env::var("REALM_ID")?.parse()?;
        let bind_ip = self.auth_db.get_realm_bind_ip(realm_id).await?;
        let tcp_listener = TcpListener::bind(bind_ip).await?;
        let mut incoming_connections = tcp_listener.incoming();

        while let Some(tcp_stream) = incoming_connections.next().await {
            println!("new connection!");
            let stream = tcp_stream?;
            let socket_wrapped = Arc::new(RwLock::new(stream));
            let client_lock = Arc::new(RwLock::new(Client::new(socket_wrapped.clone())));
            let client_id = client_lock.read().await.id;
            
            {
                let mut hashmap = self.clients.write().await;
                hashmap.insert(client_id, client_lock.clone());
            }

            let realm_seed = self.realm_seed;
            let packet_channel_for_client = packet_handle_sender.clone();

            client_lock.read().await.send_auth_challenge(realm_seed)
                .await
                .unwrap_or_else(|e| {
                    println!("Error while sending auth challenge: {:?}", e);
                    return;
                });

            task::spawn(async move {
                handle_incoming_packets(client_id, socket_wrapped, packet_channel_for_client)
                    .await
                    .unwrap_or_else(|e| {
                        println!("Error while handling packet {:?}", e);
                    });
            });
        }

        Ok(())
    }
}

async fn handle_incoming_packets(client_id: u64, socket: Arc<RwLock<TcpStream>>, packet_channel: Sender<PacketToHandle>) -> Result<()>
{
    let mut buf = vec![0u8; 1024];
    let mut read_length;
    loop
    {
        {
            let mut write_socket = socket.write().await;
            read_length = write_socket.read(&mut buf).await?;
            if read_length == 0
            {
                println!("disconnect");
                write_socket.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
        }
        let header = super::packet::read_header(&buf, read_length, false)?;

        println!("Opcode = {:?}, length = {}", header.get_cmd(), header.length);
        packet_channel.send(PacketToHandle { client_id, header })?;

        /*if header.get_cmd()? == Opcodes::CMSG_AUTH_SESSION
          {
          self.handle_auth_session(&buf).await?
          }*/
    }

    Ok(())
}


