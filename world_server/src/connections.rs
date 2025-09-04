//! Realm (world server) inbound connection acceptor.
//!
//! High-level design:
//! - The world server listens for game client TCP connections whose public endpoint
//!   (IP:port) is stored in the auth database; this lets operations change bind
//!   addresses without rebuilding binaries or distributing config files.
//! - `REALM_ID` (env var) selects the row in the auth DB so multiple realm
//!   processes can share the same code and differ only by environment.
//! - The accept loop is kept tiny: resolve bind address once, then continuously
//!   accept and hand off each socket to a per-connection task.
//! - Per-connection work is executed in detached async tasks so a slow or faulty
//!   client never stalls accepting new ones.
//! - A `flume::Sender<ClientEvent>` is cloned per connection to decouple raw IO
//!   from higher-level session / game state management; this keeps the acceptor
//!   ignorant of protocol details.
//! - Public wrapper logs and swallows errors so a transient failure (e.g. DB
//!   lookup race, ephemeral bind issue) does not panic the entire server.
//!
//! The goal is resilience and operational flexibility: configuration comes from
//! the database, runtime failures are localized, and connection lifecycle logic
//! remains isolated inside the `Connection` type / client manager elsewhere.

use std::sync::Arc;

use anyhow::Result;
use smol::{net::TcpListener, stream::StreamExt};
use tracing::error;
use wrath_auth_db::AuthDatabase;

use crate::connection::{events::ClientEvent, Connection};

/// Public entry point that launches the realm connection accept loop and
/// centralizes error reporting.
pub async fn accept_realm_connections(auth_db: Arc<AuthDatabase>, client_manager_sender: flume::Sender<ClientEvent>) {
    if let Err(e) = accept_realm_connections_impl(auth_db, client_manager_sender).await {
        error!("Error in realm_socket::accept_realm_connections: {e:?}");
    }
}

/// Internal implementation of the accept loop.
async fn accept_realm_connections_impl(auth_db: Arc<AuthDatabase>, client_manager_sender: flume::Sender<ClientEvent>) -> Result<()> {
    let realm_id: i32 = std::env::var("REALM_ID")?.parse()?;
    let bind_ip = auth_db.get_realm_bind_ip(realm_id).await?;
    let tcp_listener = TcpListener::bind(bind_ip).await?;
    let mut incoming_connections = tcp_listener.incoming();

    while let Some(tcp_stream) = incoming_connections.next().await {
        let connection = Connection::new(tcp_stream?, client_manager_sender.clone());
        smol::spawn(connection.run(auth_db.clone())).detach();
    }

    Ok(())
}
