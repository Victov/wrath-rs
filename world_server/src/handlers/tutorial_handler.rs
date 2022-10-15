use crate::prelude::*;
use crate::{character::*, packet::ServerMessageExt};

pub async fn send_tutorial_flags(character: &Character) -> Result<()> {
    wow_world_messages::wrath::SMSG_TUTORIAL_FLAGS {
        tutorial_data: character.tutorial_flags.flag_data,
    }
    .astd_send_to_character(character)
    .await
}

/*
pub async fn handle_cmsg_tutorial_flag(client_manager: &ClientManager, packet: &PacketToHandle) -> Result<()> {
    let client = client_manager.get_authenticated_client(packet.client_id).await?;
    let character_lock = client.get_active_character().await?;

    let tut_flag_index: usize = {
        let mut reader = std::io::Cursor::new(&packet.payload);
        reader.read_u32::<LittleEndian>()?.try_into()?
    };

    let mut character = character_lock.write().await;
    character.tutorial_flags.set_bit(tut_flag_index, true);
    trace!("Handled tutorial flag, flags are now: {:?}", character.tutorial_flags.flag_data);
    Ok(())
}*/
