#![allow(dead_code)]

use super::map_object::MapObject;
use crate::prelude::*;

pub struct MapCell {}

impl MapCell {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn push_object(&mut self, _object: &mut impl MapObject) -> Result<()> {
        Ok(())
    }
}
