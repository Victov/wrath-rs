use crate::client::Client;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use crate::world::World;
use crate::ClientManager;
use crate::{character::*, packet::ServerMessageExt};
use wow_world_messages::wrath::{
    CacheMask, CMSG_REQUEST_ACCOUNT_DATA, CMSG_UPDATE_ACCOUNT_DATA, SMSG_ACCOUNT_DATA_TIMES, SMSG_UPDATE_ACCOUNT_DATA,
    SMSG_UPDATE_ACCOUNT_DATA_COMPLETE,
};
use wrath_auth_db::DBAccountData;
use wrath_realm_db::RealmDatabase;

pub async fn handle_csmg_ready_for_account_data_times(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;

    let account_id = {
        client
            .data
            .read()
            .await
            .account_id
            .ok_or_else(|| anyhow!("Failed to get account_id but client was authenticated. This should never happen"))?
    };

    let mut db_account_data;
    loop {
        db_account_data = client_manager.auth_db.get_account_data(account_id).await?;
        if db_account_data.is_empty() {
            create_empty_account_data_rows(client_manager, account_id).await?;
            continue;
        }
        break;
    }
    let db_account_data = db_account_data;
    send_account_wide_account_data_times(&client, &db_account_data).await
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

//Don't call directly but instead call send_account_wide_account_data_times or
//send_character_account_data_times
async fn send_account_data_times(client: &Client, mask: CacheMask, masked_data: impl Into<Vec<u32>>) -> Result<()> {
    let unix_time = {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
    };

    SMSG_ACCOUNT_DATA_TIMES {
        unix_time,
        unknown1: 1,
        mask,
        data: masked_data.into(),
    }
    .astd_send_to_client(client)
    .await?;

    Ok(())
}

async fn send_account_wide_account_data_times(client: &Client, data: &Vec<DBAccountData>) -> Result<()> {
    let mask = CacheMask::GlobalCache as u32;
    let mut masked_data = vec![];
    for row in data {
        if mask & (1 << row.data_type) > 0 {
            masked_data.push(row.time as u32);
        }
    }

    send_account_data_times(client, CacheMask::GlobalCache, masked_data).await
}

pub async fn send_character_account_data_times(realm_database: &RealmDatabase, character: &Character) -> Result<()> {
    let client = character
        .client
        .upgrade()
        .ok_or_else(|| anyhow!("couldn't upgrade client from character"))?;

    let data = realm_database.get_character_account_data(character.guid.get_low_part()).await?;

    let mask = CacheMask::PerCharacterCache as u32;
    let mut masked_data = vec![];
    for row in data {
        if mask & (1 << row.data_type) > 0 {
            masked_data.push(row.time as u32);
        }
    }

    send_account_data_times(&client, CacheMask::PerCharacterCache, masked_data).await
}

pub async fn handle_csmg_update_account_data(
    client_manager: &ClientManager,
    client_id: u64,
    world: &World,
    data: &CMSG_UPDATE_ACCOUNT_DATA,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;

    if 1 << data.data_type & CacheMask::GlobalCache as u32 > 0 {
        let account_id = client
            .data
            .read()
            .await
            .account_id
            .ok_or_else(|| anyhow!("Failed to get account_id from client even though authenticated"))?;
        client_manager
            .auth_db
            .update_account_data(
                account_id,
                data.unix_time,
                data.data_type as u8,
                data.decompressed_size,
                data.compressed_data.as_slice(),
            )
            .await?;
    } else if let Some(character_lock) = &client.data.read().await.active_character {
        let character_id = character_lock.read().await.guid.get_low_part();
        world
            .get_realm_database()
            .update_character_account_data(
                character_id,
                data.unix_time,
                data.data_type as u8,
                data.decompressed_size,
                &data.compressed_data,
            )
            .await?;
    }

    SMSG_UPDATE_ACCOUNT_DATA_COMPLETE {
        data_type: data.data_type,
        unknown1: 0,
    }
    .astd_send_to_client(client)
    .await
}

pub async fn handle_cmsg_request_account_data(
    client_manager: &ClientManager,
    client_id: u64,
    world: &World,
    data: &CMSG_REQUEST_ACCOUNT_DATA,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;

    let (decompressed_size, account_data_bytes) = {
        if 1 << data.data_type & CacheMask::GlobalCache as u32 > 0 {
            let account_id = client
                .data
                .read()
                .await
                .account_id
                .ok_or_else(|| anyhow!("Account had no account_id even though it is authenticated"))?;

            let db_data = client_manager.auth_db.get_account_data_of_type(account_id, data.data_type as u8).await?;
            if let Some(bytes) = db_data.data {
                (db_data.decompressed_size, bytes)
            } else {
                (0, vec![])
            }
        } else if let Some(active_character_lock) = &client.data.read().await.active_character {
            let character_id = active_character_lock.read().await.guid.get_low_part();
            let db_data = world
                .get_realm_database()
                .get_character_account_data_of_type(character_id, data.data_type as u8)
                .await?;
            if let Some(bytes) = db_data.data {
                (db_data.decompressed_size, bytes)
            } else {
                (0, vec![])
            }
        } else {
            bail!("Requested account data for active character but no character is logged in as active.");
        }
    };

    SMSG_UPDATE_ACCOUNT_DATA {
        data_type: data.data_type,
        decompressed_size,
        compressed_data: account_data_bytes,
    }
    .astd_send_to_client(client)
    .await
}
