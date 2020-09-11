use std::sync::mpsc::{Receiver};
use std::sync::{Arc};
use anyhow::Result;
use super::packet::{ClientPacketHeader};
use super::client::{Client};

pub struct PacketToHandle
{
    pub header: ClientPacketHeader,
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

    pub async fn handle_queue(&self) -> Result<()>
    {
        while let Ok(packet) = self.receive_channel.try_recv()
        {
            println!("message received to handle: {:?}", packet.header.get_cmd()?);
        }

        Ok(())
    }
}
