use super::client::Client;
use crate::{
    character::{character_manager::CharacterManager, *},
    connection::{events::ServerEvent, Connection},
    prelude::*,
    world::{game_object::GameObject, World},
};
use smol::prelude::*;
use std::pin::Pin;
use wow_world_messages::wrath::ServerMessage;

pub trait ServerMessageExt: ServerMessage {
    fn astd_send_to_connection<'life0, 'life1, 'async_trait>(
        &'life0 self,
        connection: &'life1 mut Connection,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: Sync + 'async_trait,
    {
        Box::pin(async move {
            self.astd_write_encrypted_server(&mut connection.stream, connection.encryption.as_mut().unwrap())
                .await?;
            Ok(())
        })
    }
}
impl<T> ServerMessageExt for T where T: ServerMessage {}

impl ServerEvent {
    pub async fn send_to_all_in_range(
        self,
        character: &Character,
        character_manager: &CharacterManager,
        include_self: bool,
        world: &World,
    ) -> Result<()> {
        if world.get_instance_manager().try_get_map_for_character(character).is_some() {
            let in_range_guids = character.get_in_range_guids();
            for guid in in_range_guids {
                let in_range_character = character_manager.get_character(guid)?;
                self.send_to_character(in_range_character).await?;
            }
            if include_self {
                self.send_to_character(character).await?;
            }
        } else {
            warn!("Trying to send packet to all in range, but this character is not on a map");
        }
        Ok(())
    }

    pub async fn send_to_character(&self, character: &Character) -> Result<()> {
        character.connection_sender.send_async(self.clone()).await?;
        Ok(())
    }

    pub async fn send_to_client(&self, client: &Client) -> Result<()> {
        client.connection_sender.send_async(self.clone()).await?;
        Ok(())
    }
}
