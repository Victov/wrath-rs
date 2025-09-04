use crate::character::*;
use crate::connection::events::ServerEvent;
use crate::prelude::*;
use wow_world_messages::wrath::FactionInitializer;
use wow_world_messages::wrath::SMSG_INITIALIZE_FACTIONS;

const NUM_FACTIONS: u32 = 128;

pub async fn send_faction_list(character: &Character) -> Result<()> {
    let factions = (0..NUM_FACTIONS).map(|_| FactionInitializer::default()).collect();
    ServerEvent::InitializeFactions(SMSG_INITIALIZE_FACTIONS { factions })
        .send_to_character(character)
        .await
}
