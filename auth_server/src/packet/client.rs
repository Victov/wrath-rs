use crate::packet::consts::*;
use crate::packet::{BytesError, PacketReader};
use byte::ctx::{Bytes, Str};
use byte::{BytesExt, LE};
use std::convert::TryInto;
use wow_srp::{PROOF_LENGTH, PUBLIC_KEY_LENGTH, RECONNECT_CHALLENGE_DATA_LENGTH};

#[derive(Debug)]
pub struct LogonChallenge<'a> {
    pub protocol_version: u8,
    pub size: u16,
    pub game_name: &'a str,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub build: u16,
    pub platform: &'a str,
    pub os: &'a str,
    pub locale: &'a str,
    pub timezone_bias: u32,
    pub ip: &'a [u8; 4],
    pub username: &'a str,
}

#[derive(Debug)]
pub struct LogonProof<'a> {
    pub public_key: &'a [u8; PUBLIC_KEY_LENGTH as usize],
    pub proof: &'a [u8; PROOF_LENGTH as usize],
    pub crc_hash: &'a [u8; PROOF_LENGTH as usize],
    pub number_of_keys: u8,
    pub security_flags: u8,
}

#[derive(Debug)]
pub struct ReconnectProof<'a> {
    pub proof_data: &'a [u8; RECONNECT_CHALLENGE_DATA_LENGTH as usize],
    pub proof: &'a [u8; PROOF_LENGTH as usize],
    pub crc_hash: &'a [u8; PROOF_LENGTH as usize],
    pub number_of_keys: u8,
}

#[derive(Debug)]
pub enum ClientPacket<'a> {
    LogonChallenge(LogonChallenge<'a>),
    ReconnectChallenge(LogonChallenge<'a>),
    LogonProof(LogonProof<'a>),
    ReconnectProof(ReconnectProof<'a>),
    RealmListRequest,
    NotImplemented,
}

impl<'a> PacketReader<'a> for ClientPacket<'a> {
    fn read_packet(buffer: &'a [u8]) -> anyhow::Result<ClientPacket<'a>, anyhow::Error> {
        let offset = &mut 0;
        let cmd = buffer.read_with::<u8>(offset, LE).map_err(BytesError::new)?;
        let packet = match cmd {
            CMD_AUTH_LOGON_CHALLENGE | CMD_AUTH_RECONNECT_CHALLENGE => {
                let challenge = LogonChallenge {
                    protocol_version: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                    size: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                    game_name: buffer.read_with::<&str>(offset, Str::Len(4)).map_err(BytesError::new)?,
                    major: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                    minor: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                    patch: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                    build: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                    platform: buffer.read_with(offset, Str::Len(4)).map_err(BytesError::new)?,
                    os: buffer.read_with(offset, Str::Len(4)).map_err(BytesError::new)?,
                    locale: buffer.read_with(offset, Str::Len(4)).map_err(BytesError::new)?,
                    timezone_bias: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                    ip: buffer.read_with::<&[u8]>(offset, Bytes::Len(4)).map_err(BytesError::new)?.try_into()?,
                    username: {
                        let len = buffer.read_with::<u8>(offset, LE).map_err(BytesError::new)?;
                        buffer.read_with(offset, Str::Len(len as usize)).map_err(BytesError::new)?
                    },
                };
                if cmd == CMD_AUTH_LOGON_CHALLENGE {
                    ClientPacket::LogonChallenge(challenge)
                } else {
                    ClientPacket::ReconnectChallenge(challenge)
                }
            }
            CMD_AUTH_LOGON_PROOF => {
                let proof = LogonProof {
                    public_key: buffer
                        .read_with::<&[u8]>(offset, Bytes::Len(PUBLIC_KEY_LENGTH as usize))
                        .map_err(BytesError::new)?
                        .try_into()?,
                    proof: buffer
                        .read_with::<&[u8]>(offset, Bytes::Len(PROOF_LENGTH as usize))
                        .map_err(BytesError::new)?
                        .try_into()?,
                    crc_hash: buffer
                        .read_with::<&[u8]>(offset, Bytes::Len(PROOF_LENGTH as usize))
                        .map_err(BytesError::new)?
                        .try_into()?,
                    number_of_keys: buffer.read_with::<u8>(offset, LE).map_err(BytesError::new)?,
                    security_flags: buffer.read_with(offset, LE).map_err(BytesError::new)?,
                };

                ClientPacket::LogonProof(proof)
            }
            CMD_AUTH_RECONNECT_PROOF => {
                let proof = ReconnectProof {
                    proof_data: buffer
                        .read_with::<&[u8]>(offset, Bytes::Len(RECONNECT_CHALLENGE_DATA_LENGTH as usize))
                        .map_err(BytesError::new)?
                        .try_into()?,
                    proof: buffer
                        .read_with::<&[u8]>(offset, Bytes::Len(PROOF_LENGTH as usize))
                        .map_err(BytesError::new)?
                        .try_into()?,
                    crc_hash: buffer
                        .read_with::<&[u8]>(offset, Bytes::Len(PROOF_LENGTH as usize))
                        .map_err(BytesError::new)?
                        .try_into()?,
                    number_of_keys: buffer.read_with::<u8>(offset, LE).map_err(BytesError::new)?,
                };

                ClientPacket::ReconnectProof(proof)
            }
            CMD_REALM_LIST => {
                let _padding = buffer.read_with::<u32>(offset, LE).map_err(BytesError::new)?;
                ClientPacket::RealmListRequest
            }
            _ => ClientPacket::NotImplemented,
        };

        Ok(packet)
    }
}
