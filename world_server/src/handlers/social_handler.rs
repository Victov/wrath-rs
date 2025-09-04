use std::net::SocketAddr;

use crate::character::character_manager::CharacterManager;
use crate::connection::events::ServerEvent;
use crate::prelude::*;
use crate::world::prelude::GameObject;
use crate::world::World;
use crate::{character::*, client_manager::ClientManager};

use wow_world_base::wrath::PlayerChatTag;
use wow_world_messages::wrath::{
    CMSG_MESSAGECHAT_ChatType, RelationType, SMSG_MESSAGECHAT_ChatType, CMSG_CONTACT_LIST, CMSG_JOIN_CHANNEL, CMSG_MESSAGECHAT, CMSG_SET_SELECTION,
    SMSG_CALENDAR_SEND_NUM_PENDING, SMSG_CONTACT_LIST, SMSG_MESSAGECHAT,
};

pub async fn handle_cmsg_contact_list(
    client_manager: &ClientManager,
    character_manager: &CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_CONTACT_LIST,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character(guid)?;

    let requested_social_mask = RelationType::new(packet.flags);
    send_contact_list(character, requested_social_mask).await
}

pub async fn handle_cmsg_calendar_get_num_pending(client_manager: &ClientManager, client_id: SocketAddr) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let msg = SMSG_CALENDAR_SEND_NUM_PENDING { pending_events: 0 };
    let event = ServerEvent::CalendarSendNumPending(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn send_contact_list(character: &Character, relation_mask: RelationType) -> Result<()> {
    let msg = SMSG_CONTACT_LIST {
        list_mask: relation_mask,
        relations: vec![],
    };
    ServerEvent::ContactList(msg).send_to_character(character).await
}

pub async fn handle_cmsg_set_selection(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_SET_SELECTION,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character_mut(guid)?;

    let selection = if packet.target.is_zero() { None } else { Some(packet.target) };
    character.set_selection(selection);
    Ok(())
}

pub async fn handle_cmsg_join_channel(client_manager: &ClientManager, client_id: SocketAddr, _packet: &CMSG_JOIN_CHANNEL) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let _character = client.get_active_character();

    //There are no chat systems yet. This packet is "handled" to silence the warning spam
    Ok(())
}

pub async fn handle_cmsg_messagechat(
    client_manager: &ClientManager,
    character_manager: &CharacterManager,
    world: &World,
    client_id: SocketAddr,
    packet: &CMSG_MESSAGECHAT,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character(guid)?;

    match &packet.chat_type {
        CMSG_MESSAGECHAT_ChatType::Say | CMSG_MESSAGECHAT_ChatType::Yell | CMSG_MESSAGECHAT_ChatType::Emote => {
            handle_world_proximity_message(character, character_manager, world, packet).await?
        }
        CMSG_MESSAGECHAT_ChatType::Whisper { target_player } => {
            handle_whisper(character, target_player, client_manager, character_manager, packet).await?
        }
        _ => todo!(),
    };

    Ok(())
}

//Chat messages that are meant to arrive to people nearby.
async fn handle_world_proximity_message(
    sender: &Character,
    character_manager: &CharacterManager,
    world: &World,
    packet: &CMSG_MESSAGECHAT,
) -> Result<()> {
    let chat_type = match packet.chat_type {
        CMSG_MESSAGECHAT_ChatType::Say => SMSG_MESSAGECHAT_ChatType::Say { target6: sender.get_guid() },
        CMSG_MESSAGECHAT_ChatType::Yell => SMSG_MESSAGECHAT_ChatType::Yell { target6: sender.get_guid() },
        CMSG_MESSAGECHAT_ChatType::Emote => SMSG_MESSAGECHAT_ChatType::Emote { target6: sender.get_guid() },
        _ => bail!("This is not a world chat message type"),
    };

    let tag = PlayerChatTag::None;

    ServerEvent::MessageChat(SMSG_MESSAGECHAT {
        chat_type,
        language: packet.language,
        sender: sender.get_guid(),
        flags: 0,
        message: packet.message.clone(),
        tag,
    })
    .send_to_all_in_range(sender, character_manager, true, world)
    .await
}

async fn handle_whisper(
    sender: &Character,
    receiver_name: &str,
    client_manager: &ClientManager,
    character_manager: &CharacterManager,
    packet: &CMSG_MESSAGECHAT,
) -> Result<()> {
    assert!(std::matches!(packet.chat_type, CMSG_MESSAGECHAT_ChatType::Whisper { .. }));

    if let Ok(receiving_client) = client_manager.find_client_from_active_character_name(receiver_name, character_manager) {
        let chat_type = SMSG_MESSAGECHAT_ChatType::Whisper { target6: sender.get_guid() };
        let tag = PlayerChatTag::None;

        let msg = SMSG_MESSAGECHAT {
            chat_type,
            language: packet.language,
            sender: sender.get_guid(),
            flags: 0,
            message: packet.message.clone(),
            tag,
        };
        let event = ServerEvent::MessageChat(msg);
        receiving_client.connection_sender.send_async(event).await?;
    } else {
        let msg = SMSG_MESSAGECHAT {
            chat_type: SMSG_MESSAGECHAT_ChatType::System { target6: sender.get_guid() },
            language: wow_world_base::wrath::Language::Universal,
            sender: sender.get_guid(),
            flags: 0,
            message: "No player by that name".to_string(),
            tag: PlayerChatTag::None,
        };
        let event = ServerEvent::MessageChat(msg);
        let client = client_manager.find_client_from_active_character_guid(sender.get_guid())?;
        client.connection_sender.send_async(event).await?;
    }
    Ok(())
}
