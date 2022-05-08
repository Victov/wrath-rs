use crate::character::*;
use crate::client_manager::ClientManager;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::packet_handler::PacketToHandle;
use crate::prelude::*;
use bit_field::BitArray;
use podio::{LittleEndian, ReadPodExt};
use std::convert::TryInto;

pub async fn send_tutorial_flags(character: &Character) -> Result<()> {
    use std::io::Write;

    let (header, mut writer) = create_packet(Opcodes::SMSG_TUTORIAL_FLAGS, 32);

    writer.write(&character.tutorial_flags.flag_data)?;
    send_packet_to_character(character, &header, &writer).await?;

    Ok(())
}

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
}
