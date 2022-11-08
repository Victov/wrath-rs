use wow_world_messages::wrath::CMSG_SET_ACTIONBAR_TOGGLES;

use crate::client_manager::ClientManager;
use crate::prelude::*;

pub async fn handle_csmg_set_actionbar_toggles(client_manager: &ClientManager, client_id: u64, packet: &CMSG_SET_ACTIONBAR_TOGGLES) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;
    let action_bar = packet.action_bar;

    let mut character = character_lock.write().await;
    character.set_visible_actionbar_mask(action_bar);
    Ok(())
}
