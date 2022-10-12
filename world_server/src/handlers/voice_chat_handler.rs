use crate::prelude::*;
use crate::{character::Character, packet::ServerMessageExt};
use wow_world_messages::wrath::{ComplaintStatus, SMSG_FEATURE_SYSTEM_STATUS};

pub async fn send_voice_chat_status(character: &Character) -> Result<()> {
    SMSG_FEATURE_SYSTEM_STATUS {
        complaint_status: ComplaintStatus::EnabledWithAutoIgnore,
        voice_chat_enabled: false,
    }
    .astd_send_to_character(character)
    .await
}
