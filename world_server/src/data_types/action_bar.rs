const SIZEPERSLOT: usize = 4;
const MAXSLOTS: usize = 144;

pub struct ActionBar
{
    pub data: [u8; SIZEPERSLOT * MAXSLOTS],
}

impl ActionBar
{
    pub fn new() -> Self
    {
        Self
        {
            data: [0; SIZEPERSLOT * MAXSLOTS],
        }
    }
}
