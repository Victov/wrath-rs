pub struct UpdateData {
    pub block_count: u8,
    pub data: Vec<u8>,
}

impl UpdateData {
    pub fn new() -> Self {
        Self {
            block_count: 0,
            data: Vec::new(),
        }
    }

    pub fn add_block(&mut self, block: &[u8]) {
        self.block_count += 1;
        self.data.extend_from_slice(block);
    }
}
