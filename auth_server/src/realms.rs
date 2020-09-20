use anyhow::Result;
use podio::{ReadPodExt, WritePodExt, BigEndian};
use async_std::prelude::*;
use super::constants;
use std::time::{Instant};
use wrath_auth_db::AuthDatabase;

const HEARTBEAT_TIMEOUT_SECONDS : u64 = 15;
const REALM_MAX_POPULATION : f32 = 1000.0;

pub async fn receive_realm_pings(auth_db: std::sync::Arc<AuthDatabase>) -> Result<()>
{
    let realms = (*auth_db).get_all_realms().await?;
    let socket = async_std::net::UdpSocket::bind("127.0.0.1:1234").await?;
    let mut buffer = Vec::<u8>::new();
    buffer.resize(128, 0);

    let mut latest_heartbeats = std::collections::HashMap::new();
    for realm in realms
    {
        latest_heartbeats.insert(realm.id, Instant::now());
    }
    let heartbeats_rwlock = std::sync::Arc::new(std::sync::RwLock::new(latest_heartbeats));
    let hbwrlock_copy = heartbeats_rwlock.clone();
    let auth_db_handle = auth_db.clone();
    async_std::task::spawn(async move { 
        let mut heartbeat_interval = async_std::stream::interval(std::time::Duration::from_secs(5));
        while let Some(_) = heartbeat_interval.next().await
        {
            let hashtable = hbwrlock_copy.read().unwrap().clone();
            for (&realm_id, &heartbeat) in &hashtable
            {
                if Instant::now().duration_since(heartbeat).as_secs() > HEARTBEAT_TIMEOUT_SECONDS
                {
                    (*auth_db_handle).set_realm_online_status(realm_id, false).await.unwrap_or_else(|_| {
                        println!("Couldnt set realm status to online!");
                    });
                }
            }
        }
    });

    loop
    {
        let _ = socket.recv(&mut buffer).await?;
        let mut reader = std::io::Cursor::new(&buffer);
        let cmd = reader.read_u8()?;
        if cmd == 0 //HEARTBEAT
        {
            let realm_id = reader.read_u8()?;
            let realm_population_count = reader.read_u32::<BigEndian>()?;
            let realm_pop_current : f32 = realm_population_count as f32 / REALM_MAX_POPULATION;
            (*heartbeats_rwlock.write().unwrap()).insert(realm_id as u32, Instant::now()); 
            (*auth_db).set_realm_online_status(realm_id as u32, true).await.unwrap_or_else(|e| {
                println!("Failed to set realm online: {}", e);
            });
            (*auth_db).set_realm_population(realm_id as u32, realm_pop_current).await.unwrap_or_else(|e| {
                println!("Error while writing realm population: {}", e);
            });
        }
    }
}


pub async fn handle_realmlist_request(stream : &mut async_std::net::TcpStream, logindata: &super::auth::LoginNumbers, auth_database: &std::sync::Arc<AuthDatabase>) -> Result<()>
{
    use std::io::Write;

    println!("realmlist request");
    
    let realms = (*auth_database).get_all_realms().await?;

    let realms_info  = Vec::<u8>::new();
    let mut writer = std::io::Cursor::new(realms_info);

    let account = auth_database.get_account_by_username(&logindata.username).await?;

    for realm in &realms
    {
        let mut realm_flags = realm.flags as u8;
        if realm.online == 0
        {
            realm_flags |= constants::RealmFlags::Offline as u8;
        }
        let num_characters = auth_database.get_num_characters_on_realm(account.id, realm.id).await?;

        writer.write_u8(realm.realm_type as u8)?;
        writer.write_u8(0)?; //realm locked
        writer.write_u8(realm_flags)?;
        writer.write(realm.name.as_bytes())?;
        writer.write_u8(0)?; //string terminator
        writer.write(realm.ip.as_bytes())?;
        writer.write_u8(0)?; //string terminator
        writer.write_f32::<podio::LittleEndian>(realm.population)?;
        writer.write_u8(num_characters)?; //num characters on this realm
        writer.write_u8(realm.timezone as u8)?;
        writer.write_u8(0)?;//realm.id as u8)?; 
    }

    writer.write_u8(0x10)?; //??
    writer.write_u8(0)?; //??

    let realms_info_length = writer.get_ref().len();
    let num_realms = realms.len();

    let return_packet = Vec::<u8>::new();
    let mut packet_writer = std::io::Cursor::new(return_packet);
    packet_writer.write_u8(16)?; //REALM_LIST
    packet_writer.write_u16::<podio::LittleEndian>(realms_info_length as u16 + 6)?;
    packet_writer.write_u32::<podio::LittleEndian>(0)?;
    packet_writer.write_u16::<podio::LittleEndian>(num_realms as u16)?;
    packet_writer.write(&writer.get_ref())?;

    stream.write(&packet_writer.into_inner()).await?;
    stream.flush().await?;

    Ok(())
}

