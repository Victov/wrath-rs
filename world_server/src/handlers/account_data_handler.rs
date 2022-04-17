use crate::character::*;
use crate::client::Client;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use crate::ClientManager;
use podio::{LittleEndian, ReadPodExt, WritePodExt};
use std::io::Write;
use std::sync::Arc;
use wrath_auth_db::DBAccountData;
use wrath_realm_db::RealmDatabase;

#[allow(dead_code)]
enum CacheMask {
    GlobalCache = 0x15,
    PerCharacterCache = 0xEA,
}

pub async fn handle_csmg_ready_for_account_data_times(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;

    let account_id = {
        let client = client_lock.read().await;
        if !client.is_authenticated() {
            return Err(anyhow!("Client was not authenticated"));
        }

        client
            .account_id
            .ok_or_else(|| anyhow!("Failed to get account_id but client was authenticated. This should never happen"))?
    };

    let mut db_account_data;
    loop {
        db_account_data = client_manager.auth_db.get_account_data(account_id).await?;
        if db_account_data.len() == 0 {
            create_empty_account_data_rows(client_manager, account_id).await?;
            continue;
        }
        break;
    }

    let db_account_data = db_account_data;
    {
        let client = client_lock.read().await;
        send_account_data_times(&client, &db_account_data).await?;
    }

    Ok(())
}

async fn create_empty_account_data_rows(client_manager: &ClientManager, account_id: u32) -> Result<()> {
    for i in 0..8u8 {
        //per-character data not yet implemented
        if (CacheMask::GlobalCache as u8) & (1 << i) > 0 {
            client_manager.auth_db.create_account_data(account_id, i).await?;
        }
    }
    Ok(())
}

pub async fn create_empty_character_account_data_rows(realm_database: &RealmDatabase, character_id: u32) -> Result<()> {
    let mask = CacheMask::PerCharacterCache as u8;
    for i in 0..8u8 {
        if mask & (1 << i) > 0 {
            realm_database.create_character_account_data(character_id, i).await?;
        }
    }

    Ok(())
}

async fn send_account_data_times(client: &Client, data: &Vec<DBAccountData>) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let mask = CacheMask::GlobalCache as u32;
    let (header, mut writer) = create_packet(Opcodes::SMSG_ACCOUNT_DATA_TIMES, 41);
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    writer.write_u32::<LittleEndian>(unix_time)?;
    writer.write_u8(1)?;
    writer.write_u32::<LittleEndian>(mask as u32)?;
    for row in data {
        if mask & (1 << row.data_type) > 0 {
            writer.write_u32::<LittleEndian>(row.time as u32)?;
        }
    }

    send_packet(client, &header, &writer).await?;
    Ok(())
}

pub async fn send_character_account_data_times(client_manager: &ClientManager, character: &Character) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let client_lock = character
        .client
        .upgrade()
        .ok_or_else(|| anyhow!("couldn't upgrade client from character"))?;

    let data = client_manager.realm_db.get_character_account_data(character.guid.get_low_part()).await?;

    let mask = CacheMask::PerCharacterCache as u32;
    let (header, mut writer) = create_packet(Opcodes::SMSG_ACCOUNT_DATA_TIMES, 41);
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    writer.write_u32::<LittleEndian>(unix_time)?;
    writer.write_u8(1)?;
    writer.write_u32::<LittleEndian>(mask as u32)?;
    for row in data {
        if mask & (1 << row.data_type) > 0 {
            writer.write_u32::<LittleEndian>(row.time as u32)?;
        }
    }

    {
        let client = client_lock.read().await;
        send_packet(&client, &header, &writer).await?;
    }

    Ok(())
}

pub async fn handle_csmg_update_account_data(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;
    if !client_lock.read().await.is_authenticated() {
        return Err(anyhow!("Client trying to set account data but not authenticated"));
    }

    let mut reader = std::io::Cursor::new(&packet.payload);

    let data_type = reader.read_u32::<LittleEndian>()? as u8;
    let time = reader.read_u32::<LittleEndian>()?;
    let decompressed_size = reader.read_u32::<LittleEndian>()?;
    let new_data = reader.read_exact(packet.payload.len() - 12)?;
    assert_eq!(new_data.len() + 12, packet.header.length as usize);

    let client = client_lock.read().await;
    if 1 << data_type & CacheMask::GlobalCache as u8 > 0 {
        let account_id = client
            .account_id
            .ok_or_else(|| anyhow!("Failed to get account_id from client even though authenticated"))?;
        client_manager
            .auth_db
            .update_account_data(account_id, time, data_type, decompressed_size, &new_data)
            .await?;
    } else {
        if let Some(character_lock) = &client.active_character {
            let character_id = character_lock.read().await.guid.get_low_part();
            client_manager
                .realm_db
                .update_character_account_data(character_id, time, data_type, decompressed_size, &new_data)
                .await?;
        }
    }

    let (header, mut writer) = create_packet(Opcodes::SMSG_UPDATE_ACCOUNT_DATA_COMPLETE, 8);
    writer.write_u32::<LittleEndian>(data_type as u32)?;
    writer.write_u32::<LittleEndian>(0)?;
    send_packet(&client, &header, &writer).await?;

    Ok(())
}

pub async fn handle_cmsg_request_account_data(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;

    let client = client_lock.read().await;
    if !client.is_authenticated() {
        bail!("Client is trying to get account data but is not authenticated");
    }

    let mut reader = std::io::Cursor::new(&packet.payload);
    let data_type = reader.read_u32::<LittleEndian>()?;
    let (decompressed_size, account_data_bytes) = {
        if 1 << data_type & CacheMask::GlobalCache as u8 > 0 {
            let account_id = client
                .account_id
                .ok_or_else(|| anyhow!("Account had no account_id even though it was is authenticated"))?;

            let db_data = client_manager.auth_db.get_account_data_of_type(account_id, data_type as u8).await?;
            if let Some(bytes) = db_data.data {
                (db_data.decompressed_size, bytes)
            } else {
                (0, vec![])
            }
        } else {
            if let Some(active_character_lock) = &client.active_character {
                let character_id = active_character_lock.read().await.guid.get_low_part();
                let db_data = client_manager
                    .realm_db
                    .get_character_account_data_of_type(character_id, data_type as u8)
                    .await?;
                if let Some(bytes) = db_data.data {
                    (db_data.decompressed_size, bytes)
                } else {
                    (0, vec![])
                }
            } else {
                bail!("Requested account data for active character but no character is logged in as active.");
            }
        }
    };

    let (header, mut writer) = create_packet(Opcodes::SMSG_UPDATE_ACCOUNT_DATA, account_data_bytes.len() + 8);
    writer.write_u32::<LittleEndian>(data_type)?;
    writer.write_u32::<LittleEndian>(decompressed_size)?;
    writer.write(&account_data_bytes)?;
    send_packet(&client, &header, &writer).await
}
