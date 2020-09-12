use super::PacketToHandle;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use super::super::ClientManager;
use super::super::client::{ClientState};

pub async fn handle_cmsg_auth_session(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
{
    use podio::{ReadPodExt, LittleEndian};
    use std::io::{BufRead, Seek, SeekFrom};

    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;
    if client.client_state != ClientState::PreLogin
    {
        return Err(anyhow!("Client sent auth session but was already logged in"));
        //Disconnect hacker?
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
    let _client_seed = reader.read_u32::<LittleEndian>()?;

    reader.seek(SeekFrom::Current(20))?; //Skip unknown data

    let _client_digest = reader.read_exact(20)?;
    let _decompressed_addon_data_length = reader.read_u32::<LittleEndian>()?;
    let _compressed_addon_data = reader.read_exact(packet.header.length as usize - reader.position() as usize - 4)?;
    let db_account = client_manager.auth_db.get_account_by_username(&name).await?;
    println!("user {} has sessionkey {}", db_account.username, db_account.sessionkey);

    Ok(())
}
