use crate::character::Character;
use crate::client::{Client, ClientState};
use crate::client_manager::ClientManager;
use crate::packet::*;
use crate::prelude::*;
use podio::{LittleEndian, ReadPodExt};
use std::sync::Arc;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::wrath_header::ProofSeed;
use wow_world_messages::wrath::{
    Addon, BillingPlanFlags, RealmSplitState, SMSG_AUTH_RESPONSE_WorldResult, CMSG_AUTH_SESSION, CMSG_PING, CMSG_REALM_SPLIT, SMSG_ADDON_INFO,
    SMSG_AUTH_RESPONSE, SMSG_CLIENTCACHE_VERSION, SMSG_LOGIN_SETTIMESPEED, SMSG_LOGOUT_CANCEL_ACK, SMSG_LOGOUT_COMPLETE, SMSG_LOGOUT_RESPONSE,
    SMSG_PONG, SMSG_REALM_SPLIT, SMSG_TUTORIAL_FLAGS,
};
use wrath_auth_db::AuthDatabase;

pub async fn handle_cmsg_auth_session(client: &Client, proof_seed: ProofSeed, packet: &CMSG_AUTH_SESSION, auth_db: Arc<AuthDatabase>) -> Result<()> {
    if client.is_authenticated().await {
        client.disconnect().await?;
        warn!("duplicate login rejected!");
        bail!("Client sent auth session but was already logged in");
    }

    info!("User {} connecting with buildnumber {}", packet.username, packet.client_build);

    let db_account = match auth_db.get_account_by_username(&packet.username).await? {
        Some(c) => c,
        None => return Err(anyhow!("Account doesnt exist!")),
    };

    let mut sess_key: [u8; 40] = [0u8; 40];
    let db_session_key = hex::decode(db_account.sessionkey)?;
    assert_eq!(db_session_key.len(), 40);
    sess_key.copy_from_slice(db_session_key.as_slice());

    let client_encryption = proof_seed.into_header_crypto(
        &NormalizedString::new(&packet.username).unwrap(),
        sess_key,
        packet.client_proof,
        packet.client_seed,
    );

    if client_encryption.is_err() {
        SMSG_AUTH_RESPONSE {
            result: SMSG_AUTH_RESPONSE_WorldResult::AuthReject,
        }
        .astd_send_to_client(client)
        .await?;

        async_std::task::sleep(std::time::Duration::from_secs(2)).await;
        bail!("Failed auth attempt, rejecting");
    }

    //Set the crypto of the client for use from now on
    {
        let (encrypt, decrypt) = client_encryption.unwrap().split();
        let mut encryption = client.encryption.lock().await;
        *encryption = Some(encrypt);
        let mut decryption = client.decryption.lock().await;
        *decryption = Some(decrypt);
    }

    SMSG_AUTH_RESPONSE {
        result: SMSG_AUTH_RESPONSE_WorldResult::AuthOk {
            billing_flags: BillingPlanFlags::empty(),
            billing_rested: 0,
            billing_time: 0,
            expansion: wow_world_messages::wrath::Expansion::WrathOfTheLichLing,
        },
    }
    .astd_send_to_client(client)
    .await?;

    //Handle full world queuing here

    let addon_info = &packet.addon_info;
    let mut addon_reader = std::io::Cursor::new(addon_info);
    let num_addons = addon_reader.read_u32::<LittleEndian>()?;
    info!("num addons = {}", num_addons);
    let mut addons: Vec<Addon> = Vec::new();
    addons.reserve(num_addons as usize);

    for _ in 0..num_addons {
        use std::io::BufRead;

        let mut addon_name_buf = Vec::new();
        addon_reader.read_until(0, &mut addon_name_buf)?;
        addon_name_buf.truncate(addon_name_buf.len() - 1);
        let addon_name = String::from_utf8(addon_name_buf)?;
        let _addon_has_signature = addon_reader.read_u8()? == 1;
        let addon_crc = addon_reader.read_u32::<LittleEndian>()?;
        let _addon_extra_crc = addon_reader.read_u32::<LittleEndian>()?;
        let uses_diffent_public_key = addon_crc != 0x4C1C776D; //Blizzard addon CRC

        addons.push(Addon {
            addon_type: 2,
            uses_crc: 1,
            uses_diffent_public_key,
            unknown1: 0,
            unknown2: 0,
        });

        if uses_diffent_public_key {
            warn!("Unhandled non-blizzard addon: {}", addon_name);
            //Write blizzard public key
        }
    }

    //TODO: wow_world_messages needs changes to NOT write the size of the addon vec before writing
    //the addon vec, it corrupts the packet. Probably a skip-serialize tag that can be added to the
    //wowm file to the number_of_addons field to indicate the array size, but NOT write it into the final packet
    SMSG_ADDON_INFO { addons }.astd_send_to_client(client).await?;
    SMSG_CLIENTCACHE_VERSION { version: 0 }.astd_send_to_client(client).await?;

    send_tutorial_flags(client).await?;

    let mut client_data = client.data.write().await;
    client_data.client_state = ClientState::CharacterSelection;
    client_data.account_id = Some(db_account.id);

    Ok(())
}

async fn send_tutorial_flags(client: &Client) -> Result<()> {
    SMSG_TUTORIAL_FLAGS { tutorial_data: [0; 8] }.astd_send_to_client(client).await
}

pub async fn handle_cmsg_realm_split(client_manager: &ClientManager, client_id: u64, packet: &CMSG_REALM_SPLIT) -> Result<()> {
    let client = client_manager.get_client(client_id).await?;
    SMSG_REALM_SPLIT {
        realm_id: packet.realm_id,
        state: RealmSplitState::Normal,
        split_date: "01/01/01".into(),
    }
    .astd_send_to_client(client)
    .await
}

pub async fn handle_cmsg_ping(client_manager: &ClientManager, client_id: u64, packet: &CMSG_PING) -> Result<()> {
    let client = client_manager.get_client(client_id).await?;
    SMSG_PONG {
        sequence_id: packet.sequence_id,
    }
    .astd_send_to_client(client)
    .await
}

pub async fn send_login_set_time_speed(character: &Character) -> Result<()> {
    SMSG_LOGIN_SETTIMESPEED {
        //TODO: Use chrono for this, removed because of trait not satisfied
        datetime: wow_world_messages::DateTime::new(23, wow_world_messages::Month::July, 15, wow_world_messages::Weekday::Saturday, 12, 12),
        timescale: 0.01667f32,
        unknown1: 0,
    }
    .astd_send_to_character(character)
    .await
}

#[derive(Eq, PartialEq, Debug)]
pub enum LogoutState {
    None,
    Pending(std::time::Duration),
    Executing,
    ReturnToCharSelect,
}

pub async fn handle_cmsg_logout_request(client_manager: &ClientManager, client_id: u64) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;

    let (result, speed) = {
        let character_lock = client.get_active_character().await?;
        let mut character = character_lock.write().await;
        character.try_logout().await?
    };

    SMSG_LOGOUT_RESPONSE { result, speed }.astd_send_to_client(client).await
}

pub async fn handle_cmsg_logout_cancel(client_manager: &ClientManager, client_id: u64) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;
    let mut character = character_lock.write().await;
    character.cancel_logout().await?;
    SMSG_LOGOUT_CANCEL_ACK {}.astd_send_to_client(client).await
}

pub async fn send_smsg_logout_complete(character: &Character) -> Result<()> {
    SMSG_LOGOUT_COMPLETE {}.astd_send_to_character(character).await
}
