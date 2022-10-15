use crate::character::*;
use crate::packet::ServerMessageExt;
use crate::prelude::*;

use wow_world_messages::wrath::{RelationType, SMSG_CONTACT_LIST};

pub async fn send_contact_list(character: &Character, relation_mask: RelationType) -> Result<()> {
    SMSG_CONTACT_LIST {
        list_mask: relation_mask,
        relations: vec![],
    }
    .astd_send_to_character(character)
    .await
}
