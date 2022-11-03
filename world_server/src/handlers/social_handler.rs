use crate::packet::ServerMessageExt;
use crate::prelude::*;
use crate::{character::*, client_manager::ClientManager};

use wow_world_messages::wrath::{RelationType, CMSG_JOIN_CHANNEL, CMSG_SET_SELECTION, SMSG_CONTACT_LIST};

pub async fn send_contact_list(character: &Character, relation_mask: RelationType) -> Result<()> {
    SMSG_CONTACT_LIST {
        list_mask: relation_mask,
        relations: vec![],
    }
    .astd_send_to_character(character)
    .await
}

pub async fn handle_csmg_set_selection(client_manager: &ClientManager, client_id: u64, packet: &CMSG_SET_SELECTION) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let mut character = character_lock.write().await;
    let selection = if packet.target.is_zero() { None } else { Some(packet.target) };
    character.set_selection(selection);
    Ok(())
}

pub async fn handle_cmsg_join_channel(client_manager: &ClientManager, client_id: u64, _packet: &CMSG_JOIN_CHANNEL) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let _character_lock = client.get_active_character().await?;

    //There are no chat systems yet. This packet is "handled" to silence the warning spam
    Ok(())
}
