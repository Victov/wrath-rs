use anyhow::{Result};

pub struct World
{

}

impl World
{
    pub fn new() -> Self
    {
        Self
        {

        }
    }

    pub async fn tick(&self, delta_time: f32) -> Result<()>
    {
        Ok(())
    }
}
