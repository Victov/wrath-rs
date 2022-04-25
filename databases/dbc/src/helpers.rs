use std::mem::size_of;

use anyhow::Result;

pub trait ReadSkip {
    fn skip_bytes(&mut self, num_bytes: usize) -> Result<()>;
    fn skip<T>(&mut self, num: usize) -> Result<()> {
        self.skip_bytes(num * size_of::<T>())
    }
}

impl<R: podio::ReadPodExt> ReadSkip for R {
    fn skip_bytes(&mut self, num_bytes: usize) -> Result<()> {
        let _truncate = self.read_exact(num_bytes)?;
        Ok(())
    }
}
