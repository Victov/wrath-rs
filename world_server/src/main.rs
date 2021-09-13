use anyhow::Result;
use async_std::task;
use wrath_auth_db::AuthDatabase;
use wrath_realm_db::RealmDatabase;

mod auth;
mod character;
mod client;
mod client_manager;
mod constants;
mod data_types;
mod guid;
mod handlers;
mod opcodes;
mod packet;
mod packet_handler;
mod world;
mod wowcrypto;

use client_manager::ClientManager;
use packet_handler::{PacketHandler, PacketToHandle};

#[async_std::main]
async fn main() -> Result<()> {
    println!("Starting World Server");
    dotenv::dotenv().ok();

    let auth_database = AuthDatabase::new(&std::env::var("AUTH_DATABASE_URL")?).await?;
    let auth_database_ref = std::sync::Arc::new(auth_database);

    let realm_database = RealmDatabase::new(&std::env::var("REALM_DATABASE_URL")?).await?;
    let realm_database_ref = std::sync::Arc::new(realm_database);

    task::spawn(auth::auth_server_heartbeats());

    let world = std::sync::Arc::new(world::World::new());

    let (sender, receiver) = std::sync::mpsc::channel::<PacketToHandle>();
    let realm_packet_handler = PacketHandler::new(receiver, world.clone());

    let client_manager = std::sync::Arc::new(ClientManager::new(auth_database_ref.clone(), realm_database_ref.clone(), world.clone()));
    let client_manager_for_acceptloop = client_manager.clone();

    task::spawn(async move {
        client_manager_for_acceptloop
            .accept_realm_connections(sender)
            .await
            .unwrap_or_else(|e| println!("Error in realm_socket::accept_realm_connections: {:?}", e))
    });

    let desired_timestep_sec: f32 = 1.0 / 10.0;
    let mut previous_loop_total: f32 = desired_timestep_sec;
    loop {
        let before = std::time::Instant::now();
        realm_packet_handler.handle_queue(&client_manager).await.unwrap_or_else(|e| {
            println!("Error while handling packet: {}", e);
        });
        world.tick(previous_loop_total).await?;
        let after = std::time::Instant::now();
        let update_duration = after.duration_since(before);
        if update_duration.as_secs_f32() < desired_timestep_sec {
            task::sleep(std::time::Duration::from_secs_f32(desired_timestep_sec - update_duration.as_secs_f32())).await;
        } else {
            println!("Warning: Too long tick to keep up with desired timestep!");
        }
        previous_loop_total = std::time::Instant::now().duration_since(before).as_secs_f32();
    }
}
