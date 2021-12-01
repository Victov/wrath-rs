use crate::packet::consts::*;
use crate::packet::utils::{read_sized_bytes, read_sized_string, read_sized_string_with_len_field_u8};
use crate::packet::PacketReader;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

// This could use some derive marcos just to skip the impl part,
// but its magic to me how to write that derive marco

//FIXME(wmxd): dont use Vec and String, use &[u8] and &str with lifetimes
#[derive(Debug)]
pub struct LogonChallenge {
    pub protocol_version: u8,
    pub size: u16,
    pub game_name: String,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub build: u16,
    pub platform: String,
    pub os: String,
    pub locale: String,
    pub timezone_bias: u32,
    pub ip: Vec<u8>,
    pub username: String,
    pub reconnect: bool,
}

#[derive(Debug)]
pub struct LogonProof {
    pub public_key: Vec<u8>,
    pub proof: Vec<u8>,
    pub crc_hash: Vec<u8>,
    pub number_of_keys: u8,
    pub security_flags: u8,
    pub reconnect: bool,
}

#[derive(Debug)]
pub enum ClientPacket {
    LogonChallenge(LogonChallenge),
    LogonProof(LogonProof),
    RealmListRequest,
    NotImplemented,
}

impl PacketReader for ClientPacket {
    fn read_packet<R>(reader: &mut R) -> Result<Self>
    where
        R: Read,
    {
        let cmd = reader.read_u8()?;
        let packet = match cmd {
            CMD_AUTH_LOGON_CHALLENGE | CMD_AUTH_RECONNECT_CHALLENGE => ClientPacket::LogonChallenge(LogonChallenge {
                protocol_version: reader.read_u8()?,
                size: reader.read_u16::<LittleEndian>()?,
                game_name: read_sized_string(reader, 4)?,
                major: reader.read_u8()?,
                minor: reader.read_u8()?,
                patch: reader.read_u8()?,
                build: reader.read_u16::<LittleEndian>()?,
                platform: read_sized_string(reader, 4)?,
                os: read_sized_string(reader, 4)?,
                locale: read_sized_string(reader, 4)?,
                timezone_bias: reader.read_u32::<LittleEndian>()?,
                ip: read_sized_bytes(reader, 4)?,
                username: read_sized_string_with_len_field_u8(reader)?,
                reconnect: cmd == CMD_AUTH_RECONNECT_CHALLENGE,
            }),
            CMD_AUTH_LOGON_PROOF | CMD_AUTH_RECONNECT_PROOF => ClientPacket::LogonProof(LogonProof {
                public_key: read_sized_bytes(reader, 32)?,
                proof: read_sized_bytes(reader, 20)?,
                crc_hash: read_sized_bytes(reader, 20)?,
                number_of_keys: reader.read_u8()?,
                security_flags: if cmd != CMD_AUTH_RECONNECT_PROOF { reader.read_u8()? } else { 0 },
                reconnect: cmd == CMD_AUTH_RECONNECT_PROOF,
            }),
            CMD_REALM_LIST => {
                let _padding = reader.read_u32::<LittleEndian>()?;
                ClientPacket::RealmListRequest
            }
            _ => ClientPacket::NotImplemented,
        };

        Ok(packet)
    }
}
