use async_std::task;
use anyhow::Result;

mod auth;

#[async_std::main]
async fn main() -> Result<()> {
    println!("Starting World Server");
    dotenv::dotenv().ok();
    
    auth::auth_server_heartbeats().await.unwrap();

    
    Ok(())
}
