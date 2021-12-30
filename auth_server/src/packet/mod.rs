pub(crate) mod client;
pub(crate) mod server;

mod consts;

use anyhow::Result;
use async_std::io::WriteExt;
use async_std::net::TcpStream;
use async_trait::async_trait;
use byte::Error;
use std::fmt::Formatter;
use std::io::{Cursor, Write};

#[derive(Debug)]
pub struct BytesError {
    error: byte::Error,
}

impl BytesError {
    pub fn new(error: byte::Error) -> Self {
        Self { error }
    }
}

impl std::fmt::Display for BytesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.error {
            Error::Incomplete => {
                write!(f, "Incomplete!")
            }
            Error::BadOffset(o) => {
                write!(f, "Bad offset! {}", o)
            }
            Error::BadInput { err } => {
                write!(f, "Bad input! {}", err)
            }
        }
    }
}

impl std::error::Error for BytesError {}

pub(crate) trait PacketReader<'a>: Sized {
    fn read_packet(buffer: &'a [u8]) -> Result<Self>;
}

pub(crate) trait PacketWriter: Sized {
    fn write_packet<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write;
}

#[async_trait]
pub(crate) trait AsyncPacketWriterExt {
    async fn write_packet<T>(&mut self, packet: T) -> Result<()>
    where
        T: PacketWriter + Send;
}

#[async_trait]
impl AsyncPacketWriterExt for TcpStream {
    async fn write_packet<T>(&mut self, packet: T) -> Result<()>
    where
        T: PacketWriter + Send,
    {
        let buf = Vec::new();
        let mut cursor = Cursor::new(buf);
        packet.write_packet(&mut cursor)?;
        self.write(&cursor.get_ref()).await?;
        self.flush().await?;
        Ok(())
    }
}
