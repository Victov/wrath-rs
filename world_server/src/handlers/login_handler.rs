use super::PacketToHandle;
use podio::{WritePodExt, ReadPodExt, LittleEndian};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use super::super::ClientManager;
use super::super::client::{Client, ClientState};
use super::super::packet::*;
use super::Opcodes;

#[allow(dead_code)]
#[derive(PartialEq)]
enum AuthResponse
{
    AuthOk = 0x0C,
    Failed = 0x0D,
    Reject = 0x0E,
    BadProof = 0x0F,
    Unavailable = 0x10,
    SystemError = 0x11,
    BillingError = 0x12,
    BillingExpired= 0x13,
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
    LockedEnforced = 0x22
}

pub async fn handle_cmsg_auth_session(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
{
    use std::io::{BufRead, Seek, SeekFrom};
    use num_bigint::BigUint;
    use num_traits::Num;
    use crypto::digest::Digest;

    let client_lock = client_manager.get_client(packet.client_id).await?;
    {
        let client = client_lock.read().await;
        if client.client_state != ClientState::PreLogin
        {
            return Err(anyhow!("Client sent auth session but was already logged in"));
            //Disconnect hacker?
        }
    }
    
    let mut reader = std::io::Cursor::new(&packet.payload);
    //reader.seek(std::io::SeekFrom::Start(6))?; //skip header
    let build_number = reader.read_u32::<LittleEndian>()?;
    let _unknown1  = reader.read_u32::<LittleEndian>()?;
    let mut name = Vec::new();
    reader.read_until(0, &mut name)?;
    name.truncate(name.len()-1);
    let name = String::from_utf8(name)?;
   
    println!("user {} connecting with buildnumber {}", name, build_number);
    
    let _unknown2 = reader.read_u32::<LittleEndian>()?;
    let client_seed = reader.read_u32::<LittleEndian>()?;

    reader.seek(SeekFrom::Current(20))?; //Skip unknown data

    let client_digest = reader.read_exact(20)?;
    let decompressed_addon_data_length = reader.read_u32::<LittleEndian>()?;
    let compressed_addon_data = reader.read_exact(packet.header.length as usize - reader.position() as usize - 4)?;
    let db_account = client_manager.auth_db.get_account_by_username(&name).await?;
    //Handle db_account failed to fetch with reponse
    
    let sessionkey = BigUint::from_str_radix(&db_account.sessionkey, 16)?;
    let sesskey_bytes = sessionkey.to_bytes_le();

    assert_eq!(sesskey_bytes.len(), 40);
    {
        let mut client = client_lock.write().await;
        client.init_crypto(sesskey_bytes.as_slice())?;
    }

    let mut sha1 = crypto::sha1::Sha1::new();
    sha1.input(&name.as_bytes());
    sha1.input(&[0u8;4]);
    sha1.input(&client_seed.to_le_bytes());
    sha1.input(&client_manager.realm_seed.to_le_bytes());
    sha1.input(&sesskey_bytes);
    let mut out_buf = [0u8;20];
    sha1.result(&mut out_buf);

    let a = BigUint::from_bytes_le(&client_digest);
    let b = BigUint::from_bytes_le(&out_buf);
    if a != b
    {
        let client = client_lock.read().await;
        send_auth_response(AuthResponse::Reject, &client).await?;
        return Err(anyhow::anyhow!("Failed auth attempt, rejecting"));
    }
    //Handle full world queuing here
    
    let client = client_lock.read().await;
    send_auth_response(AuthResponse::AuthOk, &client).await?;

    {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut addon_data_decoder = ZlibDecoder::new(compressed_addon_data.as_slice());
        let mut decompressed_addon_data = Vec::<u8>::new();
        addon_data_decoder.read_to_end(&mut decompressed_addon_data)?;
        if decompressed_addon_data.len() != decompressed_addon_data_length as usize
        {
            return Err(anyhow::anyhow!("decompressed addon data length didn't match expectation"));
        }
        //Parse decompressed addon data and send Addon packet
    }
    Ok(())
}

async fn send_auth_response(response: AuthResponse, receiver: &Client) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_AUTH_RESPONSE, 1);
    let resp_u8 = response as u8;
    writer.write_u8(resp_u8)?;
    if resp_u8 == AuthResponse::AuthOk as u8 && false //Conflicting info, no need to send this?
    {
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u8(2)?;
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u8(0)?;
    }

    send_packet(receiver, header, &writer).await?;

    Ok(())
}
