use std::sync::mpsc::{Receiver};
use anyhow::Result;
use super::client_manager::ClientManager;
use super::packet::{ClientPacketHeader};
use std::sync::Arc;
use super::handlers::*;
use super::opcodes::Opcodes;

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
            self.handle_packet(client_manager, &packet).await?;
        }

        Ok(())
    }

    async fn handle_packet(&self, client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
    {
        match packet.header.get_cmd()?
        {
            Opcodes::CMSG_AUTH_SESSION => handle_cmsg_auth_session(client_manager, packet).await,
            op => Err(anyhow::anyhow!("Unhandled opcode {:?}", op))
        }
    }
}
