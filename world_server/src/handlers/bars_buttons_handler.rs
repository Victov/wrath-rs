use wow_world_messages::wrath::{ActionButton, CMSG_SET_ACTIONBAR_TOGGLES, CMSG_SET_ACTION_BUTTON};

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

pub async fn handle_cmsg_set_action_button(client_manager: &ClientManager, client_id: u64, packet: &CMSG_SET_ACTION_BUTTON) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;
    let button_slot = packet.button;
    let action_button = ActionButton {
        action: packet.action,
        action_type: packet.action_type,
        misc: packet.misc,
    };

    let mut character = character_lock.write().await;
    character.set_action_bar_button(button_slot, action_button);
    Ok(())
}
