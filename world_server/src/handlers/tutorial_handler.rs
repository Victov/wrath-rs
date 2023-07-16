use crate::client_manager::ClientManager;
use crate::prelude::*;
use crate::{character::*, packet::ServerMessageExt};
use bit_field::BitArray;
use wow_world_messages::wrath::{CMSG_TUTORIAL_FLAG, CMSG_TUTORIAL_RESET};

pub async fn send_tutorial_flags(character: &Character) -> Result<()> {
    wow_world_messages::wrath::SMSG_TUTORIAL_FLAGS {
        tutorial_data: character.tutorial_flags.flag_data,
    }
    .astd_send_to_character(character)
    .await
}

pub async fn handle_cmsg_tutorial_flag(client_manager: &ClientManager, client_id: u64, packet: &CMSG_TUTORIAL_FLAG) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let tut_flag_index = packet.tutorial_flag as usize;

    let mut character = character_lock.write().await;
    character.tutorial_flags.set_bit(tut_flag_index, true);
    trace!("Handled tutorial flag, flags are now: {:?}", character.tutorial_flags.flag_data);
    Ok(())
}

pub async fn handle_cmsg_tutorial_reset(client_manager: &ClientManager, client_id: u64) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id).await?;
    let character_lock = client.get_active_character().await?;

    let mut character = character_lock.write().await;
    character.tutorial_flags.reset();
    trace!("Reset all tutorials for: {}", character.name);
    Ok(())
}
