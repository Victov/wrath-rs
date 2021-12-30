use super::constants;
use super::prelude::*;
use anyhow::*;
use async_std::net::TcpStream;
use std::time::Instant;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::server::{SrpProof, SrpVerifier};
use wow_srp::{PublicKey, GENERATOR, LARGE_SAFE_PRIME_LITTLE_ENDIAN, PASSWORD_VERIFIER_LENGTH, SALT_LENGTH};
use wrath_auth_db::AuthDatabase;

use crate::packet::client::{LogonChallenge, LogonProof, ReconnectProof};
use crate::packet::server::{LogonChallengeBody, LogonProofBody, ReconnectChallengeBody, ServerPacket};
use crate::state::{ActiveClients, SrpServerTime};
use crate::{AsyncPacketWriterExt, ClientState};

pub async fn handle_logon_proof_srp<'a>(
    stream: &mut TcpStream,
    logon_proof: &LogonProof<'a>,
    srp_proof: SrpProof,
    username: String,
    clients: ActiveClients,
    auth_database: std::sync::Arc<AuthDatabase>,
) -> Result<ClientState> {
    let client_public_key = match PublicKey::from_le_bytes(logon_proof.public_key) {
        Ok(key) => key,
        Err(_) => {
            reject_logon_proof(stream, AuthResult::FailIncorrectPassword).await?;
            return Err(anyhow!("Invalid client public key. This is likely a result of malformed packets."));
        }
    };

    let (srp_server, server_proof) = match srp_proof.into_server(client_public_key, *logon_proof.proof) {
        Ok(s) => s,
        Err(e) => {
            reject_logon_proof(stream, AuthResult::FailIncorrectPassword).await?;
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

    stream
        .write_packet(ServerPacket::LogonProof {
            result: AuthResult::Success as u8,
            result_padding: None,
            body: Some(LogonProofBody {
                proof: &server_proof,
                account_flag: 0,
                hardware_survey_id: 0,
                unknown_flags: 0,
            }),
        })
        .await?;

    Ok(ClientState::LogOnProof { username })
}

pub async fn handle_logon_challenge_srp<'a>(
    stream: &mut TcpStream,
    challenge: &LogonChallenge<'a>,
    auth_database: std::sync::Arc<AuthDatabase>,
) -> Result<ClientState> {
    let account = match auth_database.get_account_by_username(&challenge.username).await? {
        Some(acc) if acc.banned != 0 => {
            reject_logon_challenge(stream, constants::AuthResult::FailBanned).await?;
            return Ok(ClientState::Connected);
        }
        Some(acc) if acc.v.is_empty() || acc.s.is_empty() => {
            reject_logon_challenge(stream, constants::AuthResult::FailUnknownAccount).await?;
            return Ok(ClientState::Connected);
        }
        Some(acc) => acc,
        None => {
            reject_logon_challenge(stream, constants::AuthResult::FailUnknownAccount).await?;
            return Ok(ClientState::Connected);
        }
    };

    let username = NormalizedString::new(&account.username)?;
    let mut password_verifier: [u8; PASSWORD_VERIFIER_LENGTH as usize] = Default::default();
    let mut salt: [u8; SALT_LENGTH as usize] = Default::default();
    hex::decode_to_slice(account.v.as_bytes(), &mut password_verifier)?;
    hex::decode_to_slice(account.s.as_bytes(), &mut salt)?;
    let srp_verifier = SrpVerifier::from_database_values(username, password_verifier, salt);
    let srp_proof = srp_verifier.into_proof();
    stream
        .write_packet(ServerPacket::LogonChallenge {
            result: constants::AuthResult::Success as u8,
            body: Some(LogonChallengeBody {
                public_key: srp_proof.server_public_key(),
                generator: &[GENERATOR],
                large_safe_prime: &LARGE_SAFE_PRIME_LITTLE_ENDIAN,
                salt: srp_proof.salt(),
                checksum_salt: &[
                    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
                ],
                // https://github.com/TrinityCore/TrinityCore/blob/3.3.5/src/server/authserver/Server/AuthSession.cpp:117
                security_flags: 0,
            }),
        })
        .await?;
    Ok(ClientState::ChallengeProof {
        srp_proof,
        username: account.username,
    })
}

pub async fn handle_reconnect_challenge_srp<'a>(
    stream: &mut TcpStream,
    challenge: &LogonChallenge<'a>,
    clients: ActiveClients,
) -> Result<ClientState> {
    let challenge_data = match clients.read().await.get(challenge.username) {
        Some(c) => *c.srp_server.reconnect_challenge_data(),
        None => {
            stream
                .write_packet(ServerPacket::ReconnectChallenge {
                    result: AuthResult::FailExpired as u8,
                    body: None,
                })
                .await?;
            return Ok(ClientState::Connected);
        }
    };

    stream
        .write_packet(ServerPacket::ReconnectChallenge {
            result: AuthResult::Success as u8,
            body: Some(ReconnectChallengeBody {
                challenge_data: &challenge_data,
                checksum_salt: &[
                    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
                ],
                // https://github.com/TrinityCore/TrinityCore/blob/3.3.5/src/server/authserver/Server/AuthSession.cpp:117
            }),
        })
        .await?;

    Ok(ClientState::ReconnectProof {
        username: challenge.username.to_string(),
    })
}

pub async fn handle_reconnect_proof_srp<'a>(
    stream: &mut TcpStream,
    reconnect_proof: &ReconnectProof<'a>,
    username: String,
    clients: ActiveClients,
) -> Result<ClientState> {
    let result = match clients.write().await.get_mut(&username) {
        Some(c) => c
            .srp_server
            .verify_reconnection_attempt(*reconnect_proof.proof_data, *reconnect_proof.proof),
        None => false,
    };

    stream
        .write_packet(ServerPacket::ReconnectProof {
            result: if result {
                AuthResult::Success as u8
            } else {
                AuthResult::FailIncorrectPassword as u8
            },
            result_padding: Some(0),
        })
        .await?;

    if result {
        Ok(ClientState::LogOnProof { username })
    } else {
        return Ok(ClientState::Connected);
    }
}

async fn reject_logon_proof(stream: &mut TcpStream, result: constants::AuthResult) -> Result<()> {
    stream
        .write_packet(ServerPacket::LogonProof {
            result: result as u8,
            result_padding: Some(0),
            body: None,
        })
        .await?;
    Ok(())
}

async fn reject_logon_challenge(stream: &mut TcpStream, result: constants::AuthResult) -> Result<()> {
    stream
        .write_packet(ServerPacket::LogonChallenge {
            result: result as u8,
            body: None,
        })
        .await?;
    Ok(())
}
