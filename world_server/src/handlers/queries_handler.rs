use crate::client_manager::ClientManager;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use podio::{LittleEndian, ReadPodExt, WritePodExt};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn handle_cmsg_played_time(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;
    if !client.is_authenticated() {
        bail!("Trying to request playtime for character that isn't authenticated");
    }
    let character_lock = client
        .active_character
        .as_ref()
        .ok_or(anyhow!("Trying to obtain played time, but no character is active for this client"))?
        .clone();

    let (playtime_total, playtime_level) = {
        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
        let mut character = character_lock.write().await;
        let delta_seconds = unix_time - character.last_playtime_calculation_timestamp;
        character.seconds_played_total += delta_seconds;
        character.seconds_played_at_level += delta_seconds;
        character.last_playtime_calculation_timestamp = unix_time;
        (character.seconds_played_total, character.seconds_played_at_level)
    };

    let show_on_ui = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_u8()?
    };

    let (header, mut writer) = create_packet(Opcodes::SMSG_PLAYED_TIME, 8);
    writer.write_u32::<LittleEndian>(playtime_total)?;
    writer.write_u32::<LittleEndian>(playtime_level)?;
    writer.write_u8(show_on_ui)?;
    send_packet(&client, header, &writer).await?;
    Ok(())
}

pub async fn handle_cmsg_query_time(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
    let client_lock = client_manager.get_client(packet.client_id).await?;
    let client = client_lock.read().await;
    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    let (header, mut writer) = create_packet(Opcodes::SMSG_QUERY_TIME_RESPONSE, 8);
    writer.write_u32::<LittleEndian>(unix_time)?;
    writer.write_u32::<LittleEndian>(0)?; //unknown?
    send_packet(&client, header, &writer).await?;
    Ok(())
}
