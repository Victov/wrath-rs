use async_std::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;
use wow_srp::server::{SrpProof, SrpServer};

pub enum ClientState {
    Connected,
    ChallengeProof { srp_proof: SrpProof, username: String },
    ReconnectProof { username: String },
    LogOnProof { username: String },
}

pub struct SrpServerTime {
    pub srp_server: SrpServer,
    pub created_at: Instant,
}

pub type ActiveClients = std::sync::Arc<RwLock<HashMap<String, SrpServerTime>>>;
