use anyhow::Result;
use sqlx::MySqlPool;
use podio::{ReadPodExt, WritePodExt, BigEndian};
use async_std::prelude::*;
use super::constants;

pub async fn receive_realm_pings(database_pool: std::sync::Arc<MySqlPool>) -> Result<()>
{
    let socket = async_std::net::UdpSocket::bind("127.0.0.1:1234").await?;
    let mut buffer = Vec::<u8>::new();
    buffer.resize(128, 0);
    loop
    {
        let _length = socket.recv(&mut buffer).await?;
        let mut reader = std::io::Cursor::new(&buffer);
        let cmd = reader.read_u8()?;
        if cmd == 0 //HEARTBEAT
        {
            let realm_id = reader.read_u8()?;
            let realm_population_count = reader.read_u32::<BigEndian>()?;
            //TODO databas management and count missed heartbeats
            println!("received heartbeat from realm {} which has {} players online", realm_id, realm_population_count);
        }
    }
}


pub async fn handle_realmlist_request(stream : &mut async_std::net::TcpStream, database_pool: &MySqlPool) -> Result<()>
{
    use std::io::Write;

    println!("realmlist request");

    let realms = sqlx::query!("SELECT * FROM realms")
        .fetch_all(database_pool)
        .await?;

    let realms_info  = Vec::<u8>::new();
    let mut writer = std::io::Cursor::new(realms_info);

    for realm in &realms
    {
        let mut realm_flags = realm.flags as u8;
        if realm.online == 0
        {
            realm_flags |= constants::RealmFlags::Offline as u8;
        }

        writer.write_u8(realm.realm_type as u8)?;
        writer.write_u8(0)?; //realm locked
        writer.write_u8(realm_flags)?;
        writer.write(realm.name.as_bytes())?;
        writer.write_u8(0)?; //string terminator
        writer.write(realm.ip.as_bytes())?;
        writer.write_u8(0)?; //string terminator
        writer.write_f32::<podio::LittleEndian>(realm.population)?;
        writer.write_u8(1)?; //num characters on this realm
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

