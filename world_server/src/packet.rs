use super::client::Client;
use crate::{character::*, prelude::*};
use async_std::prelude::*;
use std::{borrow::Borrow, ops::Deref, pin::Pin};
use wow_world_messages::wrath::ServerMessage;

/*
 * TODO: refactor this to be inside ServerMessageExt
pub async fn send_packet_to_all_in_range(
    character: &Character,
    include_self: bool,
    world: &World,
    header: &ServerPacketHeader,
    payload: &Cursor<Vec<u8>>,
) -> Result<()> {
    if let Some(map) = world.get_instance_manager().try_get_map_for_character(character).await {
        let in_range_guids = character.as_world_object().unwrap().get_in_range_guids();
        for guid in in_range_guids {
            let object_lock = map
                .try_get_object(guid)
                .await
                .ok_or_else(|| anyhow!("GUID is in range, but not a valid object"))?
                .upgrade()
                .ok_or_else(|| anyhow!("object was on the map, but is no longer valid to send packets to"))?;
            let read_obj = object_lock.read().await;
            if let Some(in_range_character) = read_obj.as_character() {
                send_packet_to_character(in_range_character, header, payload).await?;
            }
        }
        if include_self {
            send_packet_to_character(character, header, payload).await?;
        }
    } else {
        warn!("Trying to send packet to all in range, but this character is not on a map");
    }

    Ok(())
} */

pub trait ServerMessageExt: ServerMessage {
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
            self.astd_write_encrypted_server(
                &mut *client.write_socket.lock().await,
                client.encryption.lock().await.as_mut().unwrap().encrypter(),
            )
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
            self.astd_write_encrypted_server(
                &mut *client.write_socket.lock().await,
                client.encryption.lock().await.as_mut().unwrap().encrypter(),
            )
            .await?;
            Ok(())
        })
    }
}
impl<T> ServerMessageExt for T where T: ServerMessage {}
