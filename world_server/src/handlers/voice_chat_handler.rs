use crate::character::Character;
use crate::connection::events::ServerEvent;
use crate::prelude::*;
use wow_world_messages::wrath::{ComplaintStatus, SMSG_FEATURE_SYSTEM_STATUS};

pub async fn send_voice_chat_status(character: &Character) -> Result<()> {
    ServerEvent::FeatureSystemStatus(SMSG_FEATURE_SYSTEM_STATUS {
        complaint_status: ComplaintStatus::EnabledWithAutoIgnore,
        voice_chat_enabled: false,
    })
    .send_to_character(character)
    .await
}
