use anyhow::Result;
use crate::packet::*;
use crate::opcodes::Opcodes;
use crate::client::Client;
use podio::{WritePodExt};

pub async fn send_voice_chat_status(client: &Client, enabled: bool) -> Result<()>
{
    let (header, mut writer) = create_packet(Opcodes::SMSG_FEATURE_SYSTEM_STATUS, 1);
    writer.write_u8(2)?; //Unknown
    writer.write_u8(enabled as u8)?;
    send_packet(client, header, &writer).await
}
