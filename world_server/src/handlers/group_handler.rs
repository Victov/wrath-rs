use std::net::SocketAddr;

use wow_world_messages::wrath::SMSG_RAID_INSTANCE_INFO;

use crate::{client_manager::ClientManager, connection::events::ServerEvent, prelude::*};

pub async fn handle_cmsg_request_raid_info(client_manager: &ClientManager, client_id: SocketAddr) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;

    let msg = SMSG_RAID_INSTANCE_INFO { raid_infos: vec![] };
    let event = ServerEvent::RaidInstanceInfo(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}
