use anyhow::{anyhow, Result};
use async_std::net::TcpStream;
use tracing::info;
use std::time::Instant;

use wrath_auth_db::AuthDatabase;

use wow_srp::normalized_string::NormalizedString;
use wow_srp::server::{SrpProof, SrpVerifier};
use wow_srp::{PublicKey, GENERATOR, LARGE_SAFE_PRIME_LITTLE_ENDIAN, PASSWORD_VERIFIER_LENGTH, SALT_LENGTH};

use wow_login_messages::version_8::*;
use wow_login_messages::{all::*, ServerMessage};

use crate::state::{ActiveClients, SrpServerTime};
use crate::{ClientState};

pub async fn handle_logon_proof_srp(
    stream: &mut TcpStream,
    logon_proof: &CMD_AUTH_LOGON_PROOF_Client,
    srp_proof: SrpProof,
    username: String,
    clients: ActiveClients,
    auth_database: std::sync::Arc<AuthDatabase>,
) -> Result<ClientState> {
    let client_public_key = match PublicKey::from_le_bytes(&logon_proof.client_public_key) {
        Ok(key) => key,
        Err(_) => {
            reject_logon_proof(stream, CMD_AUTH_LOGON_PROOF_Server_LoginResult::FailIncorrectPassword).await?;
            return Err(anyhow!("Invalid client public key. This is likely a result of malformed packets."));
        }
    };

    let (srp_server, server_proof) = match srp_proof.into_server(client_public_key, logon_proof.client_proof) {
        Ok(s) => s,
        Err(e) => {
            reject_logon_proof(stream, CMD_AUTH_LOGON_PROOF_Server_LoginResult::FailIncorrectPassword).await?;
            return Err(anyhow!(e));
        }
    };

    auth_database
        .set_account_sessionkey(&username, &hex::encode(srp_server.session_key()))
        .await?;

    {
        let mut map = clients.write().await;
        map.insert(
            username.clone(),
            SrpServerTime {
                srp_server,
                created_at: Instant::now(),
            },
        );
    }

    CMD_AUTH_LOGON_PROOF_Server {
        result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::Success {
            account_flag: AccountFlag::empty(),
            hardware_survey_id: 0,
            server_proof,
            unknown_flags: 0,
        },
    }
    .astd_write(stream)
    .await?;

    Ok(ClientState::LogOnProof { username })
}

pub async fn handle_logon_challenge_srp<'a>(
    stream: &mut TcpStream,
    challenge: &CMD_AUTH_LOGON_CHALLENGE_Client,
    auth_database: std::sync::Arc<AuthDatabase>,
) -> Result<ClientState> {
    let account = match auth_database.get_account_by_username(&challenge.account_name).await? {
        Some(acc) if acc.banned != 0 => {
            reject_logon_challenge(stream, CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::FailBanned).await?;
            return Ok(ClientState::Connected);
        }
        Some(acc) if acc.v.is_empty() || acc.s.is_empty() => {
            reject_logon_challenge(stream, CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::FailUnknownAccount).await?;
            return Ok(ClientState::Connected);
        }
        Some(acc) => acc,
        None => {
            reject_logon_challenge(stream, CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::FailUnknownAccount).await?;
            return Ok(ClientState::Connected);
        }
    };

    let username = NormalizedString::from(&account.username)?;
    let mut password_verifier: [u8; PASSWORD_VERIFIER_LENGTH as usize] = Default::default();
    let mut salt: [u8; SALT_LENGTH as usize] = Default::default();

    hex::decode_to_slice(account.v.as_bytes(), &mut password_verifier)?;
    hex::decode_to_slice(account.s.as_bytes(), &mut salt)?;

    let srp_verifier = SrpVerifier::from_database_values(username, password_verifier, salt);
    let srp_proof = srp_verifier.into_proof();

    CMD_AUTH_LOGON_CHALLENGE_Server {
        result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::Success {
            crc_salt: [
                0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
            ],
            generator: vec![GENERATOR],
            large_safe_prime: Vec::from(LARGE_SAFE_PRIME_LITTLE_ENDIAN),
            salt: *srp_proof.salt(),
            // https://github.com/TrinityCore/TrinityCore/blob/3.3.5/src/server/authserver/Server/AuthSession.cpp:117
            security_flag: CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag::empty(),
            server_public_key: *srp_proof.server_public_key(),
        },
    }
    .astd_write(stream)
    .await?;

    Ok(ClientState::ChallengeProof {
        srp_proof,
        username: account.username,
    })
}

pub async fn handle_reconnect_challenge_srp<'a>(
    stream: &mut TcpStream,
    challenge: &CMD_AUTH_RECONNECT_CHALLENGE_Client,
    clients: ActiveClients,
) -> Result<ClientState> {
    let challenge_data = match clients.read().await.get(&challenge.account_name) {
        Some(c) => *c.srp_server.reconnect_challenge_data(),
        None => {
            CMD_AUTH_RECONNECT_CHALLENGE_Server {
                result: CMD_AUTH_RECONNECT_CHALLENGE_Server_LoginResult::FailUnknown0,
            }
            .astd_write(stream)
            .await?;

            return Ok(ClientState::Connected);
        }
    };

    CMD_AUTH_RECONNECT_CHALLENGE_Server {
        result: CMD_AUTH_RECONNECT_CHALLENGE_Server_LoginResult::Success {
            challenge_data,
            checksum_salt: [
                0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
            ],
        },
    }
    .astd_write(stream)
    .await?;

    Ok(ClientState::ReconnectProof {
        username: challenge.account_name.to_string(),
    })
}

pub async fn handle_reconnect_proof_srp<'a>(
    stream: &mut TcpStream,
    reconnect_proof: &CMD_AUTH_RECONNECT_PROOF_Client,
    username: String,
    clients: ActiveClients,
) -> Result<ClientState> {
    let result = match clients.write().await.get_mut(&username) {
        Some(c) => c
            .srp_server
            .verify_reconnection_attempt(reconnect_proof.proof_data, reconnect_proof.client_proof),
        None => false,
    };

    CMD_AUTH_RECONNECT_PROOF_Server {
        result: if result {
            LoginResult::Success
        } else {
            LoginResult::FailIncorrectPassword
        },
    }
    .astd_write(stream)
    .await?;

    if result {
        Ok(ClientState::LogOnProof { username })
    } else {
        Ok(ClientState::Connected)
    }
}

async fn reject_logon_proof(stream: &mut TcpStream, result: CMD_AUTH_LOGON_PROOF_Server_LoginResult) -> Result<()> {
    CMD_AUTH_LOGON_PROOF_Server { result }.astd_write(stream).await?;

    Ok(())
}

async fn reject_logon_challenge(stream: &mut TcpStream, result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult) -> Result<()> {
    CMD_AUTH_LOGON_CHALLENGE_Server { result }.astd_write(stream).await?;

    Ok(())
}
