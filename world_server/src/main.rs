use async_std::task;
use anyhow::Result;

mod auth;
mod realm_socket;
mod auth_database;
mod opcodes;
mod client;

#[async_std::main]
async fn main() -> Result<()> {
    println!("Starting World Server");
    dotenv::dotenv().ok();
    
    let auth_database = auth_database::AuthDatabase::new(&std::env::var("AUTH_DATABASE_URL")?).await?;
    let auth_database_ref = std::sync::Arc::new(auth_database);

    task::spawn(auth::auth_server_heartbeats());

    let auth_db_for_realm_socket = auth_database_ref.clone();
    task::block_on(async move {
        realm_socket::accept_realm_connections(&auth_db_for_realm_socket).await.unwrap_or_else(|e| {
            println!("Error in realm_socket::accept_realm_connections: {:?}", e)
        })
    });
    
    Ok(())
}
