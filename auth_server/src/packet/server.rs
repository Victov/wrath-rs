use crate::packet::consts::{CMD_AUTH_LOGON_CHALLENGE, CMD_AUTH_LOGON_PROOF, CMD_REALM_LIST};
use crate::packet::PacketWriter;
use crate::prelude::RealmFlags;
use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Cursor, Write};
use wrath_auth_db::AuthDatabase;

#[derive(Debug)]
pub enum ServerPacket {
    LogonChallenge {
        result: u8,
        body: Option<LogonChallengeBody>,
    },
    LogonProof {
        result: u8,
        result_padding: Option<u16>,
        body: Option<LogonProofBody>,
    },
    RealmListRequest(Vec<Realm>),
}

#[derive(Debug)]
pub struct LogonChallengeBody {
    pub public_key: Vec<u8>,
    pub generator: Vec<u8>,
    pub large_safe_prime: Vec<u8>,
    pub salt: Vec<u8>,
    pub crc_salt: Vec<u8>,
    pub security_flags: u8,
}

#[derive(Debug)]
pub struct LogonProofBody {
    pub proof: Vec<u8>,
    pub account_flag: u32,
    pub hardware_survey_id: u32,
    pub unknown_flags: u16,
}

#[derive(Debug)]
pub struct Realm {
    pub realm_type: u8,
    pub locked: u8,
    pub flags: u8,
    pub name: String,
    pub address: String,
    pub population: f32,
    pub number_of_chars: u8,
    pub realm_category: u8,
    pub realm_id: u8,
}

impl Realm {
    pub async fn from_db(auth_database: std::sync::Arc<AuthDatabase>, account_id: u32) -> Result<Vec<Self>> {
        //TODO(wmxd): it will be good idea to cache the database stuff
        //TODO(wmxd): for now it will be better select realms and number_of_chars in one database trip (eg: left join)
        let db_realms = auth_database.get_all_realms().await?;
        let mut realms = Vec::with_capacity(db_realms.len());
        for realm in db_realms {
            let num_characters = auth_database.get_num_characters_on_realm(account_id, realm.id).await?;
            let mut realm_flags = realm.flags as u8;
            if realm.online == 0 {
                realm_flags |= RealmFlags::Offline as u8;
            }

            realms.push(Realm {
                realm_type: realm.realm_type,
                locked: 0,
                flags: realm_flags,
                name: realm.name,
                address: realm.ip,
                population: realm.population,
                number_of_chars: num_characters,
                realm_category: realm.timezone,
                realm_id: 0,
            });
        }

        Ok(realms)
    }
}

impl PacketWriter for ServerPacket {
    fn write_packet<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        match self {
            ServerPacket::LogonChallenge { result, body } => {
                writer.write_u8(CMD_AUTH_LOGON_CHALLENGE)?;
                writer.write_u8(0)?; // protocol version
                writer.write_u8(*result)?;
                if let Some(body) = body {
                    writer.write(&body.public_key)?;
                    writer.write_u8(body.generator.len() as u8)?;
                    writer.write(&body.generator)?;
                    writer.write_u8(body.large_safe_prime.len() as u8)?;
                    writer.write(&body.large_safe_prime)?;
                    writer.write(&body.salt)?;
                    writer.write(&body.crc_salt)?;
                    writer.write_u8(body.security_flags)?;
                }
            }
            ServerPacket::LogonProof {
                result,
                result_padding,
                body,
            } => {
                writer.write_u8(CMD_AUTH_LOGON_PROOF)?;
                writer.write_u8(*result)?;
                if let Some(result_padding) = *result_padding {
                    writer.write_u16::<LittleEndian>(result_padding)?;
                }

                if let Some(body) = body {
                    writer.write(&body.proof)?;
                    writer.write_u32::<LittleEndian>(body.account_flag)?;
                    writer.write_u32::<LittleEndian>(body.hardware_survey_id)?;
                    writer.write_u16::<LittleEndian>(body.unknown_flags)?;
                }
            }
            ServerPacket::RealmListRequest(realms) => {
                let buf = Vec::new();
                let mut cursor = Cursor::new(buf);
                cursor.write_u32::<LittleEndian>(0)?; // Some kind of padding
                cursor.write_u16::<LittleEndian>(realms.len() as u16)?;
                for realm in realms {
                    cursor.write_u8(realm.realm_type)?;
                    cursor.write_u8(realm.locked)?;
                    cursor.write_u8(realm.flags)?;
                    cursor.write(realm.name.as_bytes())?;
                    cursor.write_u8(0)?; //string terminator
                    cursor.write(realm.address.as_bytes())?;
                    cursor.write_u8(0)?; //string terminator
                    cursor.write_f32::<LittleEndian>(realm.population)?;
                    cursor.write_u8(realm.number_of_chars)?;
                    cursor.write_u8(realm.realm_category as u8)?;
                    cursor.write_u8(realm.realm_id)?;
                }
                cursor.write_u16::<LittleEndian>(0)?; // Some kind of padding

                writer.write_u8(CMD_REALM_LIST)?;
                writer.write_u16::<LittleEndian>(cursor.get_ref().len() as u16)?;
                writer.write(&cursor.get_ref())?;
            }
        }
        Ok(())
    }
}
