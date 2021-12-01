use anyhow::*;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task;
use std::io::Cursor;
use std::result::Result::Ok;
use tracing_subscriber::EnvFilter;
use wrath_auth_db::AuthDatabase;

mod auth;
mod constants;
mod packet;
mod realms;

pub mod prelude {
    pub use super::constants::*;
    pub use tracing::{error, info, trace, warn};
}

use crate::auth::{handle_logon_challenge_srp, handle_logon_proof_srp, ServerChallengeProof, ServerLogOnProof};
use crate::packet::client::ClientPacket;
use crate::packet::server::ServerPacket;
use crate::packet::{PacketReader, PacketWriter};
use crate::realms::handle_realm_list_request;
use prelude::*;

#[async_std::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("wrath=info,sqlx=warn"))
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let connect_string = &std::env::var("AUTH_DATABASE_URL")?;
    let auth_db = std::sync::Arc::new(AuthDatabase::new(&connect_string).await?);

    task::spawn(realms::receive_realm_pings(auth_db.clone()));

    let tcp_listener = TcpListener::bind("127.0.0.1:3724").await?;
    loop {
        let (stream, _) = tcp_listener.accept().await?;
        task::spawn(handle_incoming_connection(stream, auth_db.clone()));
    }
}

async fn handle_incoming_connection(mut stream: TcpStream, auth_database: std::sync::Arc<AuthDatabase>) -> Result<()> {
    info!("incoming on address {}", stream.local_addr()?.to_string());
    let mut challenge_proof: Option<ServerChallengeProof> = None;
    let mut log_on_proof: Option<ServerLogOnProof> = None;

    //TODO(wmxd): keep some state that so the packets cant come out of order
    let mut buf = [0u8; 1024];
    loop {
        let read_len = stream.read(&mut buf).await?;
        if read_len > 0 {
            let mut cursor = Cursor::new(buf);
            let packet = ClientPacket::read_packet(&mut cursor)?;
            match packet {
                ClientPacket::LogonChallenge(challenge) => {
                    //TODO(wmxd): handle reconnect should be just getting session key for username from db and generating new reconnect challenge
                    match handle_logon_challenge_srp(&mut stream, challenge, Arc::clone(&auth_database)).await {
                        Ok(srp_proof) => challenge_proof = Some(srp_proof),
                        Err(e) => error!("error {}", e),
                    }
                }
                ClientPacket::LogonProof(logon_proof) => {
                    //TODO(wmxd): handle reconnect should be just getting saved reconnect challenge and session key
                    if let Some(proof) = challenge_proof.take() {
                        match handle_logon_proof_srp(&mut stream, logon_proof, proof, Arc::clone(&auth_database)).await {
                            Ok(proof) => log_on_proof = Some(proof),
                            Err(e) => error!("error {}", e),
                        }
                    } else {
                        //TODO(wmxd): maybe just DC here?
                        let buf = Vec::new();
                        let mut cursor = Cursor::new(buf);
                        ServerPacket::LogonProof {
                            result: AuthResult::FailUnknownAccount as u8,
                            result_padding: Some(0),
                            body: None,
                        }
                        .write_packet(&mut cursor)?;

                        stream.write(&cursor.get_ref()).await?;
                        stream.flush().await?;
                    }
                }
                ClientPacket::RealmListRequest => {
                    if let Some(proof) = &log_on_proof {
                        if let Err(e) = handle_realm_list_request(&mut stream, &proof.username, Arc::clone(&auth_database)).await {
                            error!("error {}", e);
                        }
                    } else {
                        //TODO(wmxd): maybe just DC here?
                        let buf = Vec::new();
                        let mut cursor = Cursor::new(buf);
                        ServerPacket::RealmListRequest(Vec::new()).write_packet(&mut cursor)?;
                        stream.write(&cursor.get_ref()).await?;
                        stream.flush().await?;
                    }
                }
                ClientPacket::NotImplemented => {
                    warn!("unhandled {}", buf[0]);
                    return Err(anyhow!("Unhandled command header"));
                }
            }
        } else {
            info!("disconnect");
            stream.shutdown(async_std::net::Shutdown::Both)?;
            break;
        }
    }
    Ok(())
}
