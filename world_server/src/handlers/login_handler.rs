use crate::character::character_manager::CharacterManager;
use crate::character::Character;
use crate::client_manager::ClientManager;
use crate::connection::events::ServerEvent;
use crate::connection::Connection;
use crate::packet::*;
use crate::prelude::*;
use podio::{LittleEndian, ReadPodExt};
use std::net::SocketAddr;
use std::sync::Arc;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::wrath_header::ProofSeed;
use wow_world_messages::wrath::{
    Addon, BillingPlanFlags, RealmSplitState, SMSG_AUTH_RESPONSE_WorldResult, CMSG_AUTH_SESSION, CMSG_PING, CMSG_REALM_SPLIT, SMSG_ADDON_INFO,
    SMSG_AUTH_RESPONSE, SMSG_CLIENTCACHE_VERSION, SMSG_LOGIN_SETTIMESPEED, SMSG_LOGOUT_CANCEL_ACK, SMSG_LOGOUT_COMPLETE, SMSG_LOGOUT_RESPONSE,
    SMSG_PONG, SMSG_REALM_SPLIT, SMSG_TUTORIAL_FLAGS,
};
use wrath_auth_db::AuthDatabase;

pub async fn handle_cmsg_auth_session(
    connection: &mut Connection,
    proof_seed: ProofSeed,
    packet: &CMSG_AUTH_SESSION,
    auth_db: Arc<AuthDatabase>,
) -> Result<u32> {
    if connection.is_authenticated() {
        connection.disconnect().await?;
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
        .astd_send_to_connection(connection)
        .await?;

        async_io::Timer::after(std::time::Duration::from_secs(2)).await;
        bail!("Failed auth attempt, rejecting");
    }

    //Set the crypto of the client for use from now on
    {
        let (encrypt, decrypt) = client_encryption.unwrap().split();
        connection.set_crypto(encrypt, decrypt);
    }

    SMSG_AUTH_RESPONSE {
        result: SMSG_AUTH_RESPONSE_WorldResult::AuthOk {
            billing_flags: BillingPlanFlags::empty(),
            billing_rested: 0,
            billing_time: 0,
            expansion: wow_world_messages::wrath::Expansion::WrathOfTheLichLing,
        },
    }
    .astd_send_to_connection(connection)
    .await?;

    //Handle full world queuing here

    let addon_info = &packet.addon_info;
    let mut addon_reader = std::io::Cursor::new(addon_info);
    let num_addons = addon_reader.read_u32::<LittleEndian>()?;
    info!("num addons = {}", num_addons);
    let mut addons: Vec<Addon> = Vec::with_capacity(num_addons as usize);

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
    SMSG_ADDON_INFO { addons }.astd_send_to_connection(connection).await?;
    SMSG_CLIENTCACHE_VERSION { version: 0 }.astd_send_to_connection(connection).await?;

    send_tutorial_flags(connection).await?;

    Ok(db_account.id)
}

async fn send_tutorial_flags(connection: &mut Connection) -> Result<()> {
    SMSG_TUTORIAL_FLAGS { tutorial_data: [0; 8] }.astd_send_to_connection(connection).await
}

pub async fn handle_cmsg_realm_split(client_manager: &ClientManager, client_id: SocketAddr, packet: &CMSG_REALM_SPLIT) -> Result<()> {
    let client = client_manager.get_client(client_id)?;
    let msg = SMSG_REALM_SPLIT {
        realm_id: packet.realm_id,
        state: RealmSplitState::Normal,
        split_date: "01/01/01".into(),
    };
    let server_event = ServerEvent::RealmSplit(msg);
    client.connection_sender.send_async(server_event).await?;
    Ok(())
}

pub async fn handle_cmsg_ping(client_manager: &ClientManager, client_id: SocketAddr, packet: &CMSG_PING) -> Result<()> {
    let client = client_manager.get_client(client_id)?;
    let msg = SMSG_PONG {
        sequence_id: packet.sequence_id,
    };
    let event = ServerEvent::Pong(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn send_login_set_time_speed(character: &Character) -> Result<()> {
    ServerEvent::LoginSetTimeSpeed(SMSG_LOGIN_SETTIMESPEED {
        //TODO: Use chrono for this, removed because of trait not satisfied
        datetime: wow_world_messages::DateTime::new(23, wow_world_messages::Month::July, 15, wow_world_messages::Weekday::Saturday, 12, 12),
        timescale: 0.01667f32,
        unknown1: 0,
    })
    .send_to_character(character)
    .await
}

#[derive(Eq, PartialEq, Debug)]
pub enum LogoutState {
    None,
    Pending(std::time::Duration),
    Executing,
    ReturnToCharSelect,
}

pub async fn handle_cmsg_logout_request(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;

    let (result, speed) = {
        let character = character_manager.get_character_mut(client.get_active_character())?;
        character.try_logout().await?
    };

    let msg = SMSG_LOGOUT_RESPONSE { result, speed };
    let event = ServerEvent::LogoutResponse(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_logout_cancel(
    client_manager: &mut ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;
    character.cancel_logout().await?;
    let msg = SMSG_LOGOUT_CANCEL_ACK {};
    let event = ServerEvent::LogoutCancelAck(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn send_smsg_logout_complete(character: &Character) -> Result<()> {
    ServerEvent::LogoutComplete(SMSG_LOGOUT_COMPLETE {}).send_to_character(character).await
}
