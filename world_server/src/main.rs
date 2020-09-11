use async_std::task;
use anyhow::Result;
use wrath_auth_db::{AuthDatabase};

mod auth;
mod opcodes;
mod client;
mod packet;
mod client_manager;
mod packet_handler;

use packet_handler::{PacketToHandle, PacketHandler};
use client_manager::ClientManager;

#[async_std::main]
async fn main() -> Result<()> {
    println!("Starting World Server");
    dotenv::dotenv().ok();
    
    let auth_database = AuthDatabase::new(&std::env::var("AUTH_DATABASE_URL")?).await?;
    let auth_database_ref = std::sync::Arc::new(auth_database);

    task::spawn(auth::auth_server_heartbeats());
    
    let (sender, receiver) = std::sync::mpsc::channel::<PacketToHandle>();
    let realm_packet_handler = PacketHandler::new(receiver);
    
    let client_manager = ClientManager::new(auth_database_ref.clone());
    
    task::spawn(async move {
        client_manager.accept_realm_connections(sender).await.unwrap_or_else(|e| {
            println!("Error in realm_socket::accept_realm_connections: {:?}", e)
        })
    });

    loop
    {
        realm_packet_handler.handle_queue().await?;
        task::sleep(std::time::Duration::from_millis(100)).await;
    }
    
    Ok(())
}
