use wow_world_messages::wrath::SMSG_RAID_INSTANCE_INFO;

use crate::{client_manager::ClientManager, packet::ServerMessageExt, prelude::*};

pub async fn handle_cmsg_request_raid_info(client_manager: &ClientManager, client_id: u64) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;

    SMSG_RAID_INSTANCE_INFO { raid_infos: vec![] }.astd_send_to_client(client).await
}
