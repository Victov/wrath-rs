use crate::packet::ServerMessageExt;
use crate::prelude::*;
use crate::world::prelude::GameObject;
use crate::world::World;
use crate::{character::*, client_manager::ClientManager};

use wow_world_base::wrath::PlayerChatTag;
use wow_world_messages::wrath::{
    RelationType, SMSG_MESSAGECHAT_ChatType, CMSG_CONTACT_LIST, CMSG_JOIN_CHANNEL, CMSG_MESSAGECHAT, CMSG_SET_SELECTION,
    SMSG_CALENDAR_SEND_NUM_PENDING, SMSG_CONTACT_LIST, SMSG_MESSAGECHAT,
};

pub async fn handle_cmsg_contact_list(client_manager: &ClientManager, client_id: u64, packet: &CMSG_CONTACT_LIST) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let requested_social_mask = RelationType::new(packet.flags);
    let character = character_lock.write().await;
    send_contact_list(&*character, requested_social_mask).await
}

pub async fn handle_cmsg_calendar_get_num_pending(client_manager: &ClientManager, client_id: u64) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    SMSG_CALENDAR_SEND_NUM_PENDING { pending_events: 0 }.astd_send_to_client(client).await
}

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

pub async fn handle_cmsg_messagechat(client_manager: &ClientManager, world: &World, client_id: u64, packet: &CMSG_MESSAGECHAT) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let character = character_lock.write().await;

    let chat_type = SMSG_MESSAGECHAT_ChatType::Say {
        target6: character.get_guid(),
    };

    let tag = PlayerChatTag::None;

    SMSG_MESSAGECHAT {
        chat_type,
        language: packet.language,
        sender: character.get_guid(),
        flags: 0,
        message: packet.message.clone(),
        tag,
    }
    .astd_send_to_all_in_range(&*character, true, world)
    .await?;

    info!("Player {} chatted {}", character.name, packet.message);
    Ok(())
}
