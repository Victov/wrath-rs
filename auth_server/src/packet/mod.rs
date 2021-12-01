pub(crate) mod client;
pub(crate) mod server;

mod consts;
mod utils;

use anyhow::Result;
use std::io::{Read, Write};

pub(crate) trait PacketReader: Sized {
    fn read_packet<R>(reader: &mut R) -> Result<Self>
    where
        R: Read;
}

pub(crate) trait PacketWriter: Sized {
    fn write_packet<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write;
}
