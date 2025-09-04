use std::net::SocketAddr;

use crate::character::{character_manager::CharacterManager, *};
use crate::client_manager::ClientManager;
use crate::connection::events::ServerEvent;
use crate::prelude::*;
use bit_field::BitArray;
use wow_world_messages::wrath::{CMSG_TUTORIAL_FLAG, SMSG_TUTORIAL_FLAGS};

pub async fn send_tutorial_flags(character: &Character) -> Result<()> {
    ServerEvent::TutorialFlags(SMSG_TUTORIAL_FLAGS {
        tutorial_data: character.tutorial_flags.flag_data,
    })
    .send_to_character(character)
    .await
}

pub async fn handle_cmsg_tutorial_flag(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
    packet: &CMSG_TUTORIAL_FLAG,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let character = client.get_active_character();
    let character = character_manager.get_character_mut(character)?;

    let tut_flag_index = packet.tutorial_flag as usize;

    character.tutorial_flags.set_bit(tut_flag_index, true);
    trace!("Handled tutorial flag, flags are now: {:?}", character.tutorial_flags.flag_data);
    Ok(())
}

pub async fn handle_cmsg_tutorial_reset(
    client_manager: &ClientManager,
    character_manager: &mut CharacterManager,
    client_id: SocketAddr,
) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;
    let guid = client.get_active_character();
    let character = character_manager.get_character_mut(guid)?;

    character.tutorial_flags.reset();
    trace!("Reset all tutorials for: {}", character.name);
    Ok(())
}
