use crate::character::Character;
use crate::client::{Client, ClientState};
use crate::client_manager::ClientManager;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use podio::{LittleEndian, ReadPodExt, WritePodExt};
use std::sync::Arc;

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

pub async fn handle_cmsg_auth_session(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    use crypto::digest::Digest;
    use num_bigint::BigUint;
    use num_traits::Num;
    use std::io::{BufRead, Seek, SeekFrom};

    let client_lock = client_manager.get_client(packet.client_id).await?;
    {
        let client = client_lock.read().await;
        if client.is_authenticated() {
            return Err(anyhow!("Client sent auth session but was already logged in"));
            //Disconnect hacker?
        }
    }

    let mut reader = std::io::Cursor::new(&packet.payload);
    let build_number = reader.read_u32::<LittleEndian>()?;
    let _unknown1 = reader.read_u32::<LittleEndian>()?;
    let mut name = Vec::new();
    reader.read_until(0, &mut name)?;
    name.truncate(name.len() - 1);
    let name = String::from_utf8(name)?;

    info!("User {} connecting with buildnumber {}", name, build_number);

    let _unknown2 = reader.read_u32::<LittleEndian>()?;
    let client_seed = reader.read_u32::<LittleEndian>()?;

    reader.seek(SeekFrom::Current(20))?; //Skip unknown data

    let client_digest = reader.read_exact(20)?;
    let decompressed_addon_data_length = reader.read_u32::<LittleEndian>()?;
    let compressed_addon_data = reader.read_exact(packet.header.length as usize - reader.position() as usize)?;
    let db_account = client_manager.auth_db.get_account_by_username(&name).await?;
    //Handle db_account failed to fetch with reponse

    let sesskey_bytes = hex::decode(&db_account.sessionkey)?;
    assert_eq!(sesskey_bytes.len(), 40);
    {
        let client = client_lock.read().await;
        client.crypto.write().await.initialize(&sesskey_bytes)?;
    }

    let mut sha1 = crypto::sha1::Sha1::new();
    sha1.input(&name.as_bytes());
    sha1.input(&[0u8; 4]);
    sha1.input(&client_seed.to_le_bytes());
    sha1.input(&client_manager.realm_seed.to_le_bytes());
    sha1.input(&sesskey_bytes);
    let mut out_buf = [0u8; 20];
    sha1.result(&mut out_buf);

    let a = BigUint::from_bytes_le(&client_digest);
    let b = BigUint::from_bytes_le(&out_buf);

    if a != b {
        let client = client_lock.read().await;
        send_auth_response(AuthResponse::Reject, &client).await?;
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;
        return Err(anyhow!("Failed auth attempt, rejecting"));
    }
    //Handle full world queuing here

    {
        let client = client_lock.read().await;
        send_auth_response(AuthResponse::AuthOk, &client).await?;
    }

    let mut decompressed_addon_data = Vec::<u8>::new();
    {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut addon_data_decoder = ZlibDecoder::new(compressed_addon_data.as_slice());
        addon_data_decoder.read_to_end(&mut decompressed_addon_data)?;
        if decompressed_addon_data.len() != decompressed_addon_data_length as usize {
            return Err(anyhow!("decompressed addon data length didn't match expectation"));
        }
    }

    let mut addon_reader = std::io::Cursor::new(decompressed_addon_data);
    let num_addons = addon_reader.read_u32::<LittleEndian>()?;

    let (addon_packet_header, mut addon_packet_writer) = create_packet(Opcodes::SMSG_ADDON_INFO, 1024);
    for _ in 0..num_addons {
        let mut addon_name_buf = Vec::new();
        addon_reader.read_until(0, &mut addon_name_buf)?;
        addon_name_buf.truncate(addon_name_buf.len() - 1);
        let addon_name = String::from_utf8(addon_name_buf)?;
        let _addon_has_signature = addon_reader.read_u8()? == 1;
        let addon_crc = addon_reader.read_u32::<LittleEndian>()?;
        let _addon_extra_crc = addon_reader.read_u32::<LittleEndian>()?;

        addon_packet_writer.write_u8(2)?; //Addontype Blizzard
        addon_packet_writer.write_u8(1)?; //Uses CRC??
        let uses_diffent_public_key = addon_crc != 0x4C1C776D; //Blizzard addon CRC
        addon_packet_writer.write_u8(if uses_diffent_public_key { 1 } else { 0 })?;
        if uses_diffent_public_key {
            warn!("Unhandled non-blizzard addon: {}", addon_name);
            //Write blizzard public key
        }
        addon_packet_writer.write_u32::<LittleEndian>(0)?;
        addon_packet_writer.write_u8(0)?;
    }
    addon_packet_writer.write_u8(0)?; //num banned addons

    {
        let client = client_lock.read().await;
        send_packet(&client, addon_packet_header, &addon_packet_writer).await?;
        send_clientcache_version(0, &client).await?;
        send_tutorial_flags(&client).await?;
    }

    {
        let mut client = client_lock.write().await;
        client.client_state = ClientState::CharacterSelection;
        client.account_id = Some(db_account.id);
    }

    Ok(())
}

async fn send_auth_response(response: AuthResponse, receiver: &Client) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_AUTH_RESPONSE, 11);
    let resp_u8 = response as u8;
    writer.write_u8(resp_u8)?;
    if resp_u8 == AuthResponse::AuthOk as u8 {
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u8(0)?;
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u8(2)?; //0= vanilla, 1=tbc, 2=wotlk
    }

    send_packet(receiver, header, &writer).await?;

    Ok(())
}

async fn send_clientcache_version(version: u32, receiver: &Client) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_CLIENTCACHE_VERSION, 4);
    writer.write_u32::<LittleEndian>(version)?;
    send_packet(receiver, header, &writer).await
}

async fn send_tutorial_flags(receiver: &Client) -> Result<()> {
    let (header, mut writer) = create_packet(Opcodes::SMSG_TUTORIAL_FLAGS, 4 * 8);
    //Each u32 is 32 bits that each indicate a tutorial message. 0 = not seen, 1 = seen.
    //Needs to be stored in the database when the client indicates that they've seen a message
    //So that it can be sent back from here. That part is todo; For now we will just let
    //The client see every tutorial.
    for _ in 0..8 {
        writer.write_u32::<LittleEndian>(0)?;
    }
    send_packet(receiver, header, &writer).await
}

#[allow(dead_code)]
enum RealmSplitState {
    Normal = 0,
    Split = 1,
    SplitPending = 2,
}

pub async fn handle_cmsg_realm_split(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    use std::io::Write;

    let realm_id = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_u32::<LittleEndian>()?
    };

    let (header, mut writer) = create_packet(Opcodes::SMSG_REALM_SPLIT, 12);
    writer.write_u32::<LittleEndian>(realm_id)?;
    writer.write_u32::<LittleEndian>(RealmSplitState::Normal as u32)?; //Realm splitting not implemented
    writer.write("01/01/01".as_bytes())?;
    writer.write_u8(0)?; //string terminator

    {
        let client_lock = client_manager.get_client(packet.client_id).await?;
        let client = client_lock.read().await;
        send_packet(&client, header, &writer).await?;
    }
    Ok(())
}

pub async fn handle_cmsg_ping(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let mut reader = std::io::Cursor::new(&packet.payload);
    let sequence = reader.read_u32::<LittleEndian>()?;
    let _latency = reader.read_u32::<LittleEndian>()?;

    let (header, mut writer) = create_packet(Opcodes::SMSG_PONG, 4);
    writer.write_u32::<LittleEndian>(sequence)?;

    let lock = client_manager.get_client(packet.client_id).await?;
    let client = lock.read().await;
    send_packet(&client, header, &writer).await?;

    Ok(())
}

pub async fn send_login_set_time_speed(character: &Character) -> Result<()> {
    use crate::data_types::WritePackedTime;

    let (header, mut writer) = create_packet(Opcodes::SMSG_LOGIN_SETTIMESPEED, 20);
    writer.write_packed_time::<LittleEndian>(&chrono::Local::now().into())?;
    writer.write_f32::<LittleEndian>(0.01667)?; //Seems to be hardcoded value
    writer.write_u32::<LittleEndian>(0)?;
    send_packet_to_character(&character, header, &writer).await?;

    Ok(())
}
