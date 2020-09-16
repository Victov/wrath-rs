//remove after implementing handling request,
//then it will be clear what's redundant
#![allow(unused_imports)]

use super::PacketToHandle;
use podio::{WritePodExt, ReadPodExt, LittleEndian};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use super::super::ClientManager;
use super::super::client::{Client, ClientState};
use super::super::packet::*;
use super::Opcodes;

pub async fn handle_csmg_ready_for_account_data_times(client_manager: &Arc<ClientManager>, packet: &PacketToHandle) -> Result<()>
{
    println!("account data times request");
    Ok(())
}
