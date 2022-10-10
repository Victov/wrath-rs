use wow_world_messages::wrath::ActionButton;

const MAXSLOTS: usize = 144;

pub struct ActionBar {
    pub data: [ActionButton; MAXSLOTS],
}

impl ActionBar {
    pub fn new() -> Self {
        Self {
            data: [ActionButton::default(); MAXSLOTS],
        }
    }
}
