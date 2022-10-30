use anyhow::{anyhow, Result};
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::RwLock;
use async_std::task;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use time::macros::format_description;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt::time::UtcTime, EnvFilter};

use wow_login_messages::version_8::opcodes::ClientOpcodeMessage;
use wrath_auth_db::AuthDatabase;

mod auth;
mod console_input;
mod constants;
mod realms;
mod state;

use crate::auth::{handle_logon_challenge_srp, handle_logon_proof_srp, handle_reconnect_challenge_srp, handle_reconnect_proof_srp};
use crate::realms::handle_realm_list_request;
use crate::state::{ActiveClients, ClientState};

#[async_std::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let timer = UtcTime::new(format_description!("[day]-[month]-[year] [hour]:[minute]:[second]"));
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("wrath=debug,sqlx=warn"))
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(timer)
        .init();

    info!("Auth server starting");
    info!("Connecting to auth database");
    let db_connect_timeout = Duration::from_secs(std::env::var("DB_CONNECT_TIMEOUT_SECONDS")?.parse()?);
    let connect_string = std::env::var("AUTH_DATABASE_URL")?;
    let auth_db = std::sync::Arc::new(AuthDatabase::new(&connect_string, db_connect_timeout).await?);
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
        let read_len = stream.peek(&mut buf).await?;
        if read_len < 1 {
            info!("disconnect");
            stream.shutdown(async_std::net::Shutdown::Both)?;
            break;
        }

        let packet = ClientOpcodeMessage::astd_read(&mut stream).await?;

        let result = match (client_state.take(), packet) {
            (_, ClientOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(challenge)) => {
                handle_logon_challenge_srp(&mut stream, &challenge, auth_database.clone()).await
            }
            (Some(ClientState::ChallengeProof { srp_proof, username }), ClientOpcodeMessage::CMD_AUTH_LOGON_PROOF(logon_proof)) => {
                handle_logon_proof_srp(&mut stream, &logon_proof, srp_proof, username, clients.clone(), auth_database.clone()).await
            }
            (_, ClientOpcodeMessage::CMD_AUTH_LOGON_PROOF(_)) => {
                info!("LogOnProof disconnect");
                stream.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
            (Some(ClientState::LogOnProof { username }), ClientOpcodeMessage::CMD_REALM_LIST(_)) => {
                handle_realm_list_request(&mut stream, username, auth_database.clone()).await
            }
            (_, ClientOpcodeMessage::CMD_REALM_LIST(_)) => {
                info!("RealmListRequest disconnect");
                stream.shutdown(async_std::net::Shutdown::Both)?;
                break;
            }
            (_, ClientOpcodeMessage::CMD_AUTH_RECONNECT_CHALLENGE(challenge)) => {
                handle_reconnect_challenge_srp(&mut stream, &challenge, clients.clone()).await
            }
            (Some(ClientState::ReconnectProof { username }), ClientOpcodeMessage::CMD_AUTH_RECONNECT_PROOF(proof)) => {
                handle_reconnect_proof_srp(&mut stream, &proof, username, clients.clone()).await
            }
            (_, ClientOpcodeMessage::CMD_AUTH_RECONNECT_PROOF(_)) => {
                info!("ReconnectProof disconnect");
                stream.shutdown(async_std::net::Shutdown::Both)?;
                break;
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
