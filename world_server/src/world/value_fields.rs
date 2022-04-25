pub use super::super::constants::unit_fields::*;
use crate::prelude::*;
use bit_field::BitArray;

pub trait HasValueFields: ValueFieldsRaw {
    fn set_object_field_f32(&mut self, field: ObjectFields, value: f32) -> Result<()> {
        self.set_field_u32(field as usize, value.to_bits())
    }

    fn set_object_field_u32(&mut self, field: ObjectFields, value: u32) -> Result<()> {
        self.set_field_u32(field as usize, value)
    }

    fn get_object_field_f32(&self, field: ObjectFields) -> Result<f32> {
        Ok(f32::from_bits(self.get_field_u32(field as usize)?))
    }

    fn set_unit_field_u32(&mut self, field: UnitFields, value: u32) -> Result<()> {
        self.set_field_u32(field as usize, value)
    }

    fn get_unit_field_u32(&self, field: UnitFields) -> Result<u32> {
        self.get_field_u32(field as usize)
    }

    fn set_byte(&mut self, field: usize, byte_index: usize, value: u8) -> Result<()> {
        let old_val = self.get_field_u32(field)?;
        let shifted_value = (value as u32) << (byte_index * 8);
        let mask = !(0xFF << (byte_index * 8));
        let new_val = old_val & mask | shifted_value;
        self.set_field_u32(field, new_val)
    }

    //Sets the bit in the UpdateMask for every field that is not zero
    //This updatemask goes to new players, who still need all information
    fn set_mask_for_create_bits(&self, mask: &mut UpdateMask) -> Result<()> {
        for i in 0..self.get_num_value_fields() {
            let val = self.get_field_u32(i)? > 0;
            mask.set_bit(i, val)?;
        }
        Ok(())
    }
}

impl<T: ValueFieldsRaw> HasValueFields for T {}

pub trait ValueFieldsRaw {
    fn set_field_u32(&mut self, field: usize, value: u32) -> Result<()>;
    fn get_field_u32(&self, field: usize) -> Result<u32>;
    fn get_num_value_fields(&self) -> usize;
    fn clear_update_mask(&mut self);
    fn get_update_mask(&self) -> &UpdateMask;
}

pub type UpdateMaskBlockType = u32;
pub const UPDATE_MASK_BLOCK_SIZE: usize = 32;

pub struct UpdateMask {
    num_fields: usize,
    data: Vec<UpdateMaskBlockType>,
}

impl UpdateMask {
    pub fn new(num_fields: usize) -> Self {
        let num_blocks = ((num_fields as f32) / (UPDATE_MASK_BLOCK_SIZE as f32)).ceil() as usize;
        let data = vec![0; num_blocks];
        Self { num_fields, data }
    }

    pub fn get_num_fields(&self) -> usize {
        self.num_fields
    }

    pub fn get_num_blocks(&self) -> usize {
        self.data.len()
    }

    pub fn set_bit(&mut self, index: usize, value: bool) -> Result<()> {
        if index >= self.num_fields {
            bail!("Bit index out of range");
        }
        self.data.set_bit(index, value);
        Ok(())
    }

    pub fn get_bit(&self, index: usize) -> Result<bool> {
        if index >= self.num_fields {
            bail!("Bit index out of range");
        }
        Ok(self.data.get_bit(index))
    }

    pub fn get_blocks(&self) -> &[UpdateMaskBlockType] {
        &self.data
    }

    pub fn clear(&mut self) {
        for i in self.data.iter_mut() {
            *i = 0;
        }
    }

    pub fn has_any_bit(&self) -> bool {
        self.data.iter().any(|a| *a > 0)
    }
}
