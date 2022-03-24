use anyhow::{anyhow, Result};
use async_std::io::ReadExt;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::RwLock;
use async_std::task;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use wrath_auth_db::AuthDatabase;

mod auth;
mod console_input;
mod constants;
mod packet;
mod realms;
mod state;

use crate::auth::{handle_logon_challenge_srp, handle_logon_proof_srp, handle_reconnect_challenge_srp, handle_reconnect_proof_srp};
use crate::packet::client::ClientPacket;
use crate::packet::{AsyncPacketWriterExt, PacketReader};
use crate::realms::handle_realm_list_request;
use crate::state::{ActiveClients, ClientState};

#[async_std::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("wrath=debug,sqlx=warn"))
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let connect_string = std::env::var("AUTH_DATABASE_URL")?;
    let auth_db = std::sync::Arc::new(AuthDatabase::new(&connect_string).await?);
    let auth_reconnect_lifetime = std::env::var("AUTH_RECONNECT_LIFETIME")
        .map(|x| x.parse::<u64>().unwrap_or(500))
        .unwrap_or(500);
    let clients = Arc::new(RwLock::new(HashMap::new()));

    task::spawn(reconnect_clients_cleaner(clients.clone(), Duration::from_secs(auth_reconnect_lifetime)));
    task::spawn(realms::receive_realm_pings(auth_db.clone()));
    task::spawn(console_input::process_console_commands(auth_db.clone()));

    let tcp_listener = TcpListener::bind("127.0.0.1:3724").await?;
    loop {
        let (stream, _) = tcp_listener.accept().await?;
        task::spawn(handle_incoming_connection(stream, clients.clone(), auth_db.clone()));
    }
}

async fn reconnect_clients_cleaner(clients: ActiveClients, timeout: Duration) -> Result<()> {
    loop {
        {
            let mut clients = clients.write().await;
            clients.retain(|_, srp_time| srp_time.created_at.elapsed() < timeout);
        }
        task::sleep(timeout).await;
    }
}

async fn handle_incoming_connection(mut stream: TcpStream, clients: ActiveClients, auth_database: std::sync::Arc<AuthDatabase>) -> Result<()> {
    info!("incoming on address {}", stream.local_addr()?.to_string());
    let mut client_state = Some(ClientState::Connected);

    let mut buf = [0u8; 1024];
    loop {
        let read_len = stream.read(&mut buf).await?;
        if read_len < 1 {
            info!("disconnect");
            stream.shutdown(async_std::net::Shutdown::Both)?;
            break;
        }
        let packet = ClientPacket::read_packet(&buf)?;

        let result = match (client_state.take(), packet) {
            (_, ClientPacket::LogonChallenge(challenge)) => handle_logon_challenge_srp(&mut stream, &challenge, auth_database.clone()).await,
            (Some(ClientState::ChallengeProof { srp_proof, username }), ClientPacket::LogonProof(logon_proof)) => {
                handle_logon_proof_srp(&mut stream, &logon_proof, srp_proof, username, clients.clone(), auth_database.clone()).await
            }
            (_, ClientPacket::LogonProof(_)) => {
                info!("LogOnProof disconnect");
                stream.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
            (Some(ClientState::LogOnProof { username }), ClientPacket::RealmListRequest) => {
                handle_realm_list_request(&mut stream, username, auth_database.clone()).await
            }
            (_, ClientPacket::RealmListRequest) => {
                info!("RealmListRequest disconnect");
                stream.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
            (_, ClientPacket::ReconnectChallenge(challenge)) => handle_reconnect_challenge_srp(&mut stream, &challenge, clients.clone()).await,
            (Some(ClientState::ReconnectProof { username }), ClientPacket::ReconnectProof(proof)) => {
                handle_reconnect_proof_srp(&mut stream, &proof, username, clients.clone()).await
            }
            (_, ClientPacket::ReconnectProof(_)) => {
                info!("ReconnectProof disconnect");
                stream.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
            (_, ClientPacket::NotImplemented) => {
                warn!("Unhandled {}", buf[0]);
                return Err(anyhow!("Unhandled command header"));
            }
        };

        match result {
            Ok(state) => client_state = Some(state),
            Err(e) => {
                error!("Error {}", e);
                info!("disconnect!");
                stream.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
        }
    }
    Ok(())
}
