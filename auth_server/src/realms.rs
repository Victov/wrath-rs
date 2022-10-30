use crate::ClientState;

use anyhow::{anyhow, Result};
use async_std::stream::StreamExt;
use byteorder::{BigEndian, ReadBytesExt};
use std::time::Instant;
use tracing::warn;

use wow_login_messages::{
    version_8::{Population, Realm, RealmCategory, RealmType, Realm_RealmFlag, CMD_REALM_LIST_Server}, ServerMessage,
};
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
        while (heartbeat_interval.next().await).is_some() {
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

async fn get_realm_list(auth_database: std::sync::Arc<AuthDatabase>, account_id: u32) -> Result<Vec<Realm>> {
    //TODO(wmxd): it will be good idea to cache the database stuff
    //TODO(wmxd): for now it will be better select realms and number_of_chars in one database trip (eg: left join)
    let db_realms = auth_database.get_all_realms().await?;
    let mut realms = Vec::with_capacity(db_realms.len());
    for realm in db_realms {
        let num_characters = auth_database.get_num_characters_on_realm(account_id, realm.id).await?;

        // TODO: Use flags from DB.
        let mut flag = Realm_RealmFlag::empty();

        if realm.online == 0 {
            flag = flag.set_OFFLINE();
        }

        let realm_type: RealmType = RealmType::try_from(realm.realm_type).unwrap_or_default();

        realms.push(Realm {
            realm_type,
            locked: 0,
            flag,
            name: realm.name,
            address: realm.ip,
            population: Population::GreenRecommended,
            number_of_characters_on_realm: num_characters,
            realm_id: 0,
            category: RealmCategory::One,
        });
    }

    Ok(realms)
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
    let realms = get_realm_list(auth_database, account.id).await?;
    CMD_REALM_LIST_Server {
        realms,
    }.astd_write(stream).await?;

    Ok(ClientState::LogOnProof { username })
}
