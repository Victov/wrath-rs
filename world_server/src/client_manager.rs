use anyhow::Result;
use async_std::task;
use async_std::net::{TcpListener};
use async_std::stream::{StreamExt};
use async_std::sync::RwLock;
use wrath_auth_db::{AuthDatabase};
use super::client::*;
use std::sync::Arc;
use std::sync::mpsc::{Sender};
use super::packet_handler::{PacketToHandle};
use rand::RngCore;

pub struct ClientManager
{
    auth_db : Arc<AuthDatabase>,
    realm_seed : u32,
}

impl ClientManager
{
    pub fn new(auth_db : Arc<AuthDatabase>) -> Self
    {
        Self
        {
            auth_db,
            realm_seed : rand::thread_rng().next_u32(),
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
            let mut client = Client::new(socket_wrapped);
            let realm_seed = self.realm_seed;
            let packet_channel_for_client = packet_handle_sender.clone();

            client.send_auth_challenge(realm_seed)
                .await
                .unwrap_or_else(|e| {
                    println!("Error while sending auth challenge: {:?}", e);
                    return;
                });

            task::spawn(async move {
                client.handle_incoming_packets(packet_channel_for_client)
                    .await
                    .unwrap_or_else(|e| {
                        println!("Error while handling packet {:?}", e);
                    });
            });
        }

        Ok(())
    }

}
