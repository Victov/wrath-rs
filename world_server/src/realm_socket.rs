use anyhow::Result;
use async_std::net::{TcpListener};
use async_std::stream::{StreamExt};
use super::auth_database::AuthDatabase;

pub async fn accept_realm_connections(auth_db : &std::sync::Arc<AuthDatabase>) -> Result<()>
{
    let realm_id : i32 = std::env::var("REALM_ID")?.parse()?;
    let bind_ip = (*auth_db).get_realm_bind_ip(realm_id).await?;
    
    let tcp_listener = TcpListener::bind(bind_ip).await?;
    let mut incoming_connections = tcp_listener.incoming();

    while let Some(tcp_stream) = incoming_connections.next().await {
        let stream = tcp_stream?;
        println!("new connection!");
    }

    Ok(())
}
    
