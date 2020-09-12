use super::PacketToHandle;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use super::super::ClientManager;
use super::super::client::{ClientState};

pub async fn handle_cmsg_auth_session(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
{
    use podio::{ReadPodExt, LittleEndian};
    use std::io::{BufRead, Seek, SeekFrom};

    for (byte, i) in packet.payload.iter().zip(0..)
    {
        println!("{} = {}", i, byte); 
    }

    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;

    if client.client_state != ClientState::PreLogin
    {
        return Err(anyhow!("Client sent auth session but was already logged in"));
        //Disconnect hacker?
    }
    
    let mut reader = std::io::Cursor::new(&packet.payload);
    reader.seek(std::io::SeekFrom::Start(6))?; //skip header
    let build_number = reader.read_u32::<LittleEndian>()?;
    let _unknown1  = reader.read_u32::<LittleEndian>()?;
    let mut name = Vec::new();
    reader.read_until(0, &mut name)?;
    let name = String::from_utf8(name)?;
   
    println!("user {} connecting from buildnumer {}", name, build_number);
    
    let _unknown2 = reader.read_u32::<LittleEndian>()?;
    let _client_seed = reader.read_u32::<LittleEndian>()?;

    reader.seek(SeekFrom::Current(20))?;

    let client_digest = reader.read_exact(20)?;
    let compressed_addon_data_length = reader.read_u32::<LittleEndian>()?;
    println!("packet length = {}", packet.header.length);
    println!("compleng {}", compressed_addon_data_length);
    //let _compressed_addon_data = reader.read_exact(compressed_addon_data_length as usize)?;
    
    println!("digest: {:?}", client_digest);
    println!("also {} bytes of addon data", compressed_addon_data_length);
    
    Ok(())
}
