use super::constants;
use super::prelude::*;
use anyhow::*;
use async_std::net::TcpStream;
use async_std::prelude::*;
use std::io::Cursor;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::server::{SrpProof, SrpServer, SrpVerifier};
use wow_srp::{PublicKey, GENERATOR, LARGE_SAFE_PRIME_LITTLE_ENDIAN, PASSWORD_VERIFIER_LENGTH, PROOF_LENGTH, PUBLIC_KEY_LENGTH, SALT_LENGTH};
use wrath_auth_db::AuthDatabase;

use crate::packet::client::{LogonChallenge, LogonProof};
use crate::packet::server::{LogonChallengeBody, LogonProofBody, ServerPacket};
use crate::packet::PacketWriter;

pub struct ServerChallengeProof {
    pub srp_proof: SrpProof,
    pub username: String,
}

pub struct ServerLogOnProof {
    pub srp_server: SrpServer,
    pub username: String,
}

//TODO: check if already logged in
pub async fn handle_logon_proof_srp(
    stream: &mut TcpStream,
    logon_proof: LogonProof,
    proof: ServerChallengeProof,
    auth_database: std::sync::Arc<AuthDatabase>,
) -> Result<ServerLogOnProof> {
    let buf = Vec::new();
    let mut cursor = Cursor::new(buf);

    let mut client_public_key = [0u8; PUBLIC_KEY_LENGTH as usize];
    client_public_key.clone_from_slice(&logon_proof.public_key[..PUBLIC_KEY_LENGTH as usize]);
    let client_public_key = match PublicKey::from_le_bytes(&client_public_key) {
        Ok(key) => key,
        Err(_) => {
            ServerPacket::LogonProof {
                result: AuthResult::FailIncorrectPassword as u8,
                result_padding: Some(0),
                body: None,
            }
            .write_packet(&mut cursor)?;
            stream.write(&cursor.get_ref()).await?;
            stream.flush().await?;
            return Err(anyhow!("Invalid client public key. This is likely a result of malformed packets."));
        }
    };
    let username = proof.username;
    let mut client_proof = [0u8; PROOF_LENGTH as usize];
    client_proof.clone_from_slice(&logon_proof.proof[..PROOF_LENGTH as usize]);
    let (s, server_proof) = match proof.srp_proof.into_server(client_public_key, client_proof) {
        Ok(s) => s,
        Err(e) => {
            ServerPacket::LogonProof {
                result: AuthResult::FailIncorrectPassword as u8,
                result_padding: Some(0),
                body: None,
            }
            .write_packet(&mut cursor)?;
            stream.write(&cursor.get_ref()).await?;
            stream.flush().await?;
            return Err(anyhow!(e));
        }
    };

    auth_database.set_account_sessionkey(&username, &hex::encode(s.session_key())).await?;

    ServerPacket::LogonProof {
        result: AuthResult::Success as u8,
        result_padding: None,
        body: Some(LogonProofBody {
            proof: server_proof.to_vec(),
            account_flag: 0,
            hardware_survey_id: 0,
            unknown_flags: 0,
        }),
    }
    .write_packet(&mut cursor)?;
    stream.write(&cursor.get_ref()).await?;
    stream.flush().await?;

    Ok(ServerLogOnProof { srp_server: s, username })
}

//TODO: check if already logged in
pub async fn handle_logon_challenge_srp(
    stream: &mut TcpStream,
    challenge: LogonChallenge,
    auth_database: std::sync::Arc<AuthDatabase>,
) -> Result<ServerChallengeProof> {
    let buf = Vec::new();
    let mut cursor = Cursor::new(buf);
    let account = match auth_database.get_account_by_username(&challenge.username).await {
        Ok(acc) if acc.banned != 0 => {
            let buf = Vec::new();
            let mut cursor = Cursor::new(buf);
            ServerPacket::LogonChallenge {
                result: constants::AuthResult::FailBanned as u8,
                body: None,
            }
            .write_packet(&mut cursor)?;
            stream.write(&cursor.get_ref()).await?;
            stream.flush().await?;
            return Err(anyhow!("Account was banned, refusing login"));
        }
        Ok(acc) => acc,
        Err(_) => {
            ServerPacket::LogonChallenge {
                result: constants::AuthResult::FailUnknownAccount as u8,
                body: None,
            }
            .write_packet(&mut cursor)?;
            stream.write(&cursor.get_ref()).await?;
            stream.flush().await?;
            return Err(anyhow!("Username not found in database"));
        }
    };

    let username = NormalizedString::new(&account.username)?;
    let s = if account.v.is_empty() || account.s.is_empty() {
        let sha_pass_bytes = hex::decode(&account.sha_pass_hash)?;
        let g = SrpVerifier::from_with_username_specific_hashed_p(username, &sha_pass_bytes);
        let v = g.password_verifier();
        let s = g.salt();
        auth_database.set_account_v_s(account.id, &hex::encode(v), &hex::encode(s)).await?;
        g
    } else {
        let mut password_verifier: [u8; PASSWORD_VERIFIER_LENGTH as usize] = Default::default();
        let mut salt: [u8; SALT_LENGTH as usize] = Default::default();
        hex::decode_to_slice(account.v.as_bytes(), &mut password_verifier)?;
        hex::decode_to_slice(account.s.as_bytes(), &mut salt)?;
        SrpVerifier::from_database_values(username, password_verifier, salt)
    };

    let proof = s.into_proof();
    ServerPacket::LogonChallenge {
        result: constants::AuthResult::Success as u8,
        body: Some(LogonChallengeBody {
            public_key: proof.server_public_key().to_vec(),
            generator: vec![GENERATOR],
            large_safe_prime: LARGE_SAFE_PRIME_LITTLE_ENDIAN.to_vec(),
            salt: proof.salt().to_vec(),
            crc_salt: vec![
                0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
            ],
            // https://github.com/TrinityCore/TrinityCore/blob/3.3.5/src/server/authserver/Server/AuthSession.cpp:117
            security_flags: 0,
        }),
    }
    .write_packet(&mut cursor)?;
    stream.write(&cursor.get_ref()).await?;
    stream.flush().await?;
    Ok(ServerChallengeProof {
        srp_proof: proof,
        username: account.username,
    })
}
