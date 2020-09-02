use anyhow::Result;
use sqlx::MySqlPool;
use podio::WritePodExt;
use async_std::prelude::*;
use super::constants;

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
        writer.write_u8(realm.realm_type as u8)?;
        writer.write_u8(0)?; //realm locked
        writer.write_u8(realm.flags as u8)?;
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

    println!("realms_info_length: {}, num_realms: {}", realms_info_length, num_realms);

    let return_packet = Vec::<u8>::new();
    let mut packet_writer = std::io::Cursor::new(return_packet);
    packet_writer.write_u8(16)?; //REALM_LIST
    packet_writer.write_u16::<podio::LittleEndian>(realms_info_length as u16 + 6)?;
    packet_writer.write_u32::<podio::LittleEndian>(0)?;
    packet_writer.write_u16::<podio::LittleEndian>(num_realms as u16)?;
    packet_writer.write(&writer.get_ref())?;

    let written_length = stream.write(&packet_writer.into_inner()).await?;
    println!("written length = {}", written_length);
    stream.flush().await?;

    Ok(())
}

