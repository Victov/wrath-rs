use std::sync::Arc;

use crate::character::Character;
use crate::client::{Client, ClientState};
use crate::client_manager::ClientManager;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use podio::{LittleEndian, ReadPodExt, WritePodExt};
use wow_srp::normalized_string::NormalizedString;
use wow_srp::wrath_header::ProofSeed;
use wow_world_messages::wrath::opcodes::ClientOpcodeMessage;
use wow_world_messages::wrath::{
    Addon, SMSG_AUTH_RESPONSE_WorldResult, ServerMessage, CMSG_AUTH_SESSION, SMSG_ADDON_INFO, SMSG_AUTH_RESPONSE, SMSG_TUTORIAL_FLAGS,
};
use wrath_auth_db::AuthDatabase;

#[allow(dead_code)]
#[derive(PartialEq)]
enum AuthResponse {
    AuthOk = 0x0C,
    Failed = 0x0D,
    Reject = 0x0E,
    BadProof = 0x0F,
    Unavailable = 0x10,
    SystemError = 0x11,
    BillingError = 0x12,
    BillingExpired = 0x13,
    VersionMismatch = 0x14,
    UnknownAccount = 0x15,
    IncorrectPass = 0x16,
    SessionExpired = 0x17,
    ServerShuttingDown = 0x18,
    AlreadyLoggedin = 0x19,
    LoginServerNotFound = 0x1A,
    WaitQueue = 0x1B,
    Banned = 0x1C,
    AlreadyOnline = 0x1D,
    NoTime = 0x1E,
    DatabaseBusy = 0x1F,
    Suspended = 0x20,
    ParentalControl = 0x21,
    LockedEnforced = 0x22,
}

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
        send_auth_response(SMSG_AUTH_RESPONSE_WorldResult::AuthReject, client).await?;
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;
        bail!("Failed auth attempt, rejecting");
    }

    //Set the crypto of the client for use from now on
    {
        let mut encryption = client.encryption.lock().await;
        *encryption = Some(client_encryption.unwrap());
    }

    //Handle full world queuing here
    send_auth_response(
        SMSG_AUTH_RESPONSE_WorldResult::AuthOk {
            billing_flags: 0,
            billing_rested: 0,
            billing_time: 0,
            expansion: wow_world_messages::wrath::Expansion::WrathOfTheLichLing,
        },
        client,
    )
    .await?;

    let mut decompressed_addon_data = Vec::<u8>::new();
    {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut addon_data_decoder = ZlibDecoder::new(packet.compressed_addon_info.as_slice());
        addon_data_decoder.read_to_end(&mut decompressed_addon_data)?;
        if decompressed_addon_data.len() != packet.decompressed_addon_info_size as usize {
            return Err(anyhow!("decompressed addon data length didn't match expectation"));
        }
    }

    let mut addon_reader = std::io::Cursor::new(decompressed_addon_data);
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
            uses_diffent_public_key: uses_diffent_public_key as u8,
        });

        if uses_diffent_public_key {
            warn!("Unhandled non-blizzard addon: {}", addon_name);
            //Write blizzard public key
        }
    }

    SMSG_ADDON_INFO { addons }
        .astd_write_encrypted_server(
            &mut *client.write_socket.lock().await,
            client.encryption.lock().await.as_mut().unwrap().encrypter(),
        )
        .await?;

    send_clientcache_version(0, client).await?;
    send_tutorial_flags(client).await?;

    {
        let mut client_data = client.data.write().await;
        client_data.client_state = ClientState::CharacterSelection;
        client_data.account_id = Some(db_account.id);
    }

    Ok(())
}

async fn send_auth_response(response: SMSG_AUTH_RESPONSE_WorldResult, client: &Client) -> Result<()> {
    SMSG_AUTH_RESPONSE { result: response }
        .astd_write_encrypted_server(
            &mut *client.write_socket.lock().await,
            client.encryption.lock().await.as_mut().unwrap().encrypter(),
        )
        .await?;

    Ok(())
}

async fn send_clientcache_version(version: u32, receiver: &Client) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_CLIENTCACHE_VERSION, 4);
    writer.write_u32::<LittleEndian>(version)?;
    send_packet(receiver, &header, &writer).await
}

async fn send_tutorial_flags(client: &Client) -> Result<()> {
    SMSG_TUTORIAL_FLAGS {
        tutorial_data0: 0,
        tutorial_data1: 0,
        tutorial_data2: 0,
        tutorial_data3: 0,
        tutorial_data4: 0,
        tutorial_data5: 0,
        tutorial_data6: 0,
        tutorial_data7: 0,
    }
    .astd_write_encrypted_server(
        &mut *client.write_socket.lock().await,
        client.encryption.lock().await.as_mut().unwrap().encrypter(),
    )
    .await?;

    Ok(())
}

#[allow(dead_code)]
enum RealmSplitState {
    Normal = 0,
    Split = 1,
    SplitPending = 2,
}

/*
pub async fn handle_cmsg_realm_split(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    use std::io::Write;

    let realm_id = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_u32::<LittleEndian>()?
    };

    let (header, mut writer) = create_packet(Opcodes::SMSG_REALM_SPLIT, 12);
    writer.write_u32::<LittleEndian>(realm_id)?;
    writer.write_u32::<LittleEndian>(RealmSplitState::Normal as u32)?; //Realm splitting not implemented
    writer.write_all("01/01/01".as_bytes())?;
    writer.write_u8(0)?; //string terminator

    let client = client_manager.get_client(packet.client_id).await?;
    send_packet(&client, &header, &writer).await?;
    Ok(())
}

pub async fn handle_cmsg_ping(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let mut reader = std::io::Cursor::new(&packet.payload);
    let sequence = reader.read_u32::<LittleEndian>()?;
    let _latency = reader.read_u32::<LittleEndian>()?;

    let (header, mut writer) = create_packet(Opcodes::SMSG_PONG, 4);
    writer.write_u32::<LittleEndian>(sequence)?;

    let client = client_manager.get_client(packet.client_id).await?;
    send_packet(&client, &header, &writer).await?;

    Ok(())
}

pub async fn send_login_set_time_speed(character: &Character) -> Result<()> {
    use crate::data::WritePackedTime;

    let (header, mut writer) = create_packet(Opcodes::SMSG_LOGIN_SETTIMESPEED, 20);
    writer.write_packed_time::<LittleEndian>(&chrono::Local::now().into())?;
    writer.write_f32::<LittleEndian>(0.01667)?; //Seems to be hardcoded value
    writer.write_u32::<LittleEndian>(0)?;
    send_packet_to_character(character, &header, &writer).await?;

    Ok(())
}

#[derive(PartialEq, Debug)]
pub enum LogoutState {
    None,
    Pending(std::time::Duration),
    Executing,
    ReturnToCharSelect,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum LogoutResult {
    Success,
    FailInCombat,
    FailFrozenByGM,
    FailJumpingOrFalling,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogoutSpeed {
    Delayed,
    Instant,
}

pub async fn handle_cmsg_logout_request(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let character_lock = client.get_active_character().await?;

    let (result, speed) = {
        let mut character = character_lock.write().await;
        character.try_logout().await?
    };

    let (header, mut writer) = create_packet(Opcodes::SMSG_LOGOUT_RESPONSE, 5);
    writer.write_u16::<LittleEndian>(result as u16)?;
    writer.write_u16::<LittleEndian>(speed as u16)?;
    send_packet(&client, &header, &writer).await
}

pub async fn handle_cmsg_logout_cancel(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let character_lock = client.get_active_character().await?;

    {
        let mut character = character_lock.write().await;
        character.cancel_logout().await?;
    }

    let (header, writer) = create_packet(Opcodes::SMSG_LOGOUT_CANCEL_ACK, 0);
    send_packet(&client, &header, &writer).await
}

pub async fn send_smsg_logout_complete(character: &Character) -> Result<()> {
    let (header, writer) = create_packet(Opcodes::SMSG_LOGOUT_COMPLETE, 0);
    send_packet_to_character(character, &header, &writer).await
}
*/
