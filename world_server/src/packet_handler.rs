use std::sync::mpsc::{Receiver};
use anyhow::Result;
use super::client_manager::ClientManager;
use super::packet::{ClientPacketHeader};
use std::sync::Arc;

pub struct PacketToHandle
{
    pub header: ClientPacketHeader,
    pub client_id: u64,
    pub payload: Vec<u8>,
}

pub struct PacketHandler
{
    receive_channel: Receiver<PacketToHandle>,
}

impl PacketHandler
{
    pub fn new(packet_receiver_channel: Receiver<PacketToHandle>) -> Self
    {
        Self
        {
            receive_channel: packet_receiver_channel,
        }
    }

    pub async fn handle_queue(&self, client_manager: &Arc<ClientManager>) -> Result<()>
    {
        while let Ok(packet) = self.receive_channel.try_recv()
        {
            println!("message received to handle: {:?}", packet.header.get_cmd()?);
            let client_lock = client_manager.get_client(packet.client_id).await?;
            let client = client_lock.read().await;
            println!("Retrieved client {}, with payload size: {} ", client.id, packet.payload.len());
        }

        Ok(())
    }
}
