use crate::client_manager::ClientManager;
use crate::data_types::{guid::*, ReadMovementInfo, WriteMovementInfo};
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use std::sync::Arc;

pub async fn handle_movement_generic(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()> {
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

    let mut reader = std::io::Cursor::new(&packet.payload);
    let guid = reader.read_guid_compressed()?;
    let movement_info = reader.read_movement_info()?;

    {
        let mut character = character_lock.write().await;
        character.process_movement(&movement_info);
    }

    let (header, mut writer) = create_packet(packet.header.get_cmd()?, 8);
    writer.write_guid_compressed(&guid)?;
    writer.write_movement_info(&movement_info)?;

    let character = character_lock.read().await;
    send_packet_to_all_in_range(&character, false, &client_manager.world, &header, &writer).await?;

    Ok(())
}
