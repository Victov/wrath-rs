use anyhow::Result;

use super::map_object::MapObject;

pub struct MapCell {}

impl MapCell {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn push_object(&mut self, object: &mut impl MapObject) -> Result<()> {
        object.set_in_cell(self);
        Ok(())
    }
}
