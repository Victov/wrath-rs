use std::net::SocketAddr;

use crate::{client_manager::ClientManager, connection::events::ServerEvent, prelude::*};
use wow_world_messages::wrath::{CMSG_GMTICKET_CREATE, SMSG_GMTICKET_GETTICKET, SMSG_GMTICKET_SYSTEMSTATUS};

pub async fn handle_cmsg_gmticket_getticket(client_manager: &ClientManager, client_id: SocketAddr) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;

    /*
    //Commented away because this adds an annoying bar to the client in the top-right corner
    //Used just for testing purposes to see if the gm ticket messages are correct, keep around for
    //reference.
    let ticket_status = wow_world_messages::wrath::SMSG_GMTICKET_GETTICKET_GmTicketStatus::HasText {
        days_since_last_updated: 1.0,
        days_since_oldest_ticket_creation: 2.0,
        days_since_ticket_creation: 0.5,
        escalation_status: wow_world_messages::wrath::GmTicketEscalationStatus::GmticketAssignedtogmStatusNotAssigned,
        id: 0,
        need_more_help: false,
        read_by_gm: false,
        text: "Wrath-rs currently does not have a functional GM ticket system. Contribute on github!".into(),
    };
    */

    let ticket_status = wow_world_messages::wrath::SMSG_GMTICKET_GETTICKET_GmTicketStatus::Default;
    let msg = SMSG_GMTICKET_GETTICKET { status: ticket_status };
    let event = ServerEvent::GMTicketGetTicket(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}

pub async fn handle_cmsg_gmticket_create(client_manager: &ClientManager, client_id: SocketAddr, _packet: &CMSG_GMTICKET_CREATE) -> Result<()> {
    let _client = client_manager.get_authenticated_client(client_id)?;

    //Creating GM tickets is unhandled, there is no system in place. This function exists to
    //prevent warning spam until a GM ticketing system is made
    Ok(())
}

pub async fn handle_cmsg_gmticket_system_status(client_manager: &ClientManager, client_id: SocketAddr) -> Result<()> {
    let client = client_manager.get_authenticated_client(client_id)?;

    let msg = SMSG_GMTICKET_SYSTEMSTATUS {
        will_accept_tickets: wow_world_messages::wrath::GmTicketQueueStatus::Disabled,
    };
    let event = ServerEvent::GMTicketSystemStatus(msg);
    client.connection_sender.send_async(event).await?;
    Ok(())
}
