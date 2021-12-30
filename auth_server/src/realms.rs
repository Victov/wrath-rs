use super::prelude::*;
use crate::packet::server::{Realm, ServerPacket};
use crate::{AsyncPacketWriterExt, ClientState};
use anyhow::{anyhow, Result};
use async_std::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};
use std::time::Instant;
use wrath_auth_db::AuthDatabase;

const HEARTBEAT_TIMEOUT_SECONDS: u64 = 15;
const REALM_MAX_POPULATION: f32 = 1000.0;

pub async fn receive_realm_pings(auth_db: std::sync::Arc<AuthDatabase>) -> Result<()> {
    let realms = (*auth_db).get_all_realms().await?;
    let socket = async_std::net::UdpSocket::bind("127.0.0.1:1234").await?;
    let mut buffer = Vec::<u8>::new();
    buffer.resize(128, 0);

    let mut latest_heartbeats = std::collections::HashMap::new();
    for realm in realms {
        latest_heartbeats.insert(realm.id, Instant::now());
    }
    let heartbeats_rwlock = std::sync::Arc::new(std::sync::RwLock::new(latest_heartbeats));
    let hbwrlock_copy = heartbeats_rwlock.clone();
    let auth_db_handle = auth_db.clone();
    async_std::task::spawn(async move {
        let mut heartbeat_interval = async_std::stream::interval(std::time::Duration::from_secs(5));
        while let Some(_) = heartbeat_interval.next().await {
            let hashtable = hbwrlock_copy.read().unwrap().clone();
            for (&realm_id, &heartbeat) in &hashtable {
                if Instant::now().duration_since(heartbeat).as_secs() > HEARTBEAT_TIMEOUT_SECONDS {
                    (*auth_db_handle).set_realm_online_status(realm_id, false).await.unwrap_or_else(|_| {
                        warn!("Couldnt set realm status to online!");
                    });
                }
            }
        }
    });

    loop {
        let _ = socket.recv(&mut buffer).await?;
        let mut reader = std::io::Cursor::new(&buffer);
        let cmd = reader.read_u8()?;
        if cmd == 0
        //HEARTBEAT
        {
            let realm_id = reader.read_u8()?;
            let realm_population_count = reader.read_u32::<BigEndian>()?;
            let realm_pop_current: f32 = realm_population_count as f32 / REALM_MAX_POPULATION;
            (*heartbeats_rwlock.write().unwrap()).insert(realm_id as u32, Instant::now());
            (*auth_db).set_realm_online_status(realm_id as u32, true).await.unwrap_or_else(|e| {
                warn!("Failed to set realm online: {}", e);
            });
            (*auth_db)
                .set_realm_population(realm_id as u32, realm_pop_current)
                .await
                .unwrap_or_else(|e| {
                    warn!("Error while writing realm population: {}", e);
                });
        }
    }
}

pub async fn handle_realm_list_request(
    stream: &mut async_std::net::TcpStream,
    username: String,
    auth_database: std::sync::Arc<AuthDatabase>,
) -> Result<ClientState> {
    let account = match auth_database.get_account_by_username(&username).await? {
        Some(acc) => acc,
        None => return Err(anyhow!("Username is not in database")),
    };
    let realms = Realm::from_db(auth_database, account.id).await?;
    stream.write_packet(ServerPacket::RealmListRequest(realms)).await?;
    Ok(ClientState::LogOnProof { username })
}
