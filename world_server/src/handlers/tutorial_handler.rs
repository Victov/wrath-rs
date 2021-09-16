use crate::character::*;
use crate::opcodes::Opcodes;
use crate::packet::*;
use crate::prelude::*;

pub async fn send_tutorial_flags(character: &Character) -> Result<()> {
    use std::io::Write;

    let (header, mut writer) = create_packet(Opcodes::SMSG_TUTORIAL_FLAGS, 32);

    writer.write(&character.tutorial_flags.flag_data)?;
    send_packet_to_character(&character, header, &writer).await?;

    Ok(())
}
