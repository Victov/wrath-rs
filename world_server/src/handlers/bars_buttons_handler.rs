use std::net::SocketAddr;

use wow_world_messages::wrath::{ActionButton, CMSG_SET_ACTIONBAR_TOGGLES, CMSG_SET_ACTION_BUTTON};

use crate::character::character_manager::CharacterManager;
use crate::client_manager::ClientManager;
use crate::prelude::*;

pub async fn handle_csmg_set_actionbar_toggles(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_SET_ACTIONBAR_TOGGLES,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;
    let action_bar = packet.action_bar;

    character.set_visible_actionbar_mask(action_bar);
    Ok(())
}

pub async fn handle_cmsg_set_action_button(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_SET_ACTION_BUTTON,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = character_manager.get_character_mut(client.get_active_character())?;
    let button_slot = packet.button;
    let action_button = ActionButton {
        action: packet.action,
        action_type: packet.action_type,
        misc: packet.misc,
    };

    character.set_action_bar_button(button_slot, action_button);
    Ok(())
}
