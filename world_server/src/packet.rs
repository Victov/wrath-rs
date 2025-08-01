use super::client::Client;
use crate::{character::*, prelude::*, world::game_object::GameObject, world::World};
use smol::prelude::*;
use std::{borrow::Borrow, pin::Pin};
use wow_world_messages::wrath::ServerMessage;

pub trait ServerMessageExt: ServerMessage {
    fn astd_send_to_all_in_range<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        character: impl Borrow<Character> + 'life1 + Send,
        include_self: bool,
        world: impl Borrow<World> + 'life2 + Send,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: Sync + 'async_trait,
    {
        Box::pin(async move {
            let character = character.borrow();
            let world = world.borrow();

            if let Some(map) = world.get_instance_manager().try_get_map_for_character(character).await {
                let in_range_guids = character.get_in_range_guids();
                for guid in in_range_guids {
                    let object_lock = map
                        .try_get_object(guid)
                        .await
                        .ok_or_else(|| anyhow!("GUID is in range, but not a valid object"))?
                        .upgrade()
                        .ok_or_else(|| anyhow!("object was on the map, but is no longer valid to send packets to"))?;
                    let read_obj = object_lock.read().await;
                    if let Some(in_range_character) = read_obj.as_character() {
                        self.astd_send_to_character(in_range_character).await?;
                    }
                }
                if include_self {
                    self.astd_send_to_character(character).await?;
                }
            } else {
                warn!("Trying to send packet to all in range, but this character is not on a map");
            }
            Ok(())
        })
    }

    fn astd_send_to_character<'life0, 'life1, 'async_trait>(
        &'life0 self,
        character: impl Borrow<Character> + 'life1 + Send,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: Sync + 'async_trait,
    {
        let client = character.borrow().client.upgrade().unwrap();
        Box::pin(async move {
            self.astd_write_encrypted_server(&mut *client.write_socket.lock().await, client.encryption.lock().await.as_mut().unwrap())
                .await?;
            Ok(())
        })
    }

    fn astd_send_to_client<'life0, 'life1, 'async_trait>(
        &'life0 self,
        client: impl Borrow<Client> + 'life1 + Send,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: Sync + 'async_trait,
    {
        Box::pin(async move {
            let client: &Client = client.borrow();
            self.astd_write_encrypted_server(&mut *client.write_socket.lock().await, client.encryption.lock().await.as_mut().unwrap())
                .await?;
            Ok(())
        })
    }
}
impl<T> ServerMessageExt for T where T: ServerMessage {}
