use crate::StringTable;
use anyhow::Result;

#[derive(Debug)]
pub struct DBCAreaTrigger;

#[derive(Debug, Clone, Copy)]
pub struct AreaTriggerBox {
    pub size_x: f32,
    pub size_y: f32,
    pub size_z: f32,
    pub orientation: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum AreaTriggerShape {
    Sphere(f32),
    Box(AreaTriggerBox),
}

#[derive(Debug)]
pub struct DBCAreaTriggerRow {
    pub id: u32,
    pub map_id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub shape: AreaTriggerShape,
}

impl DBCAreaTriggerRow {}

impl super::DBCTable for DBCAreaTrigger {
    type RowType = DBCAreaTriggerRow;

    fn get_dbc_filename() -> &'static str
    where
        Self: Sized,
    {
        "AreaTrigger.dbc"
    }
}

impl super::DBCRowType for DBCAreaTriggerRow {
    type PrimaryKeyType = u32;

    fn read_row<T: std::io::Read>(reader: &mut T, _string_table: &StringTable) -> Result<Self>
    where
        Self: Sized,
    {
        use podio::{LittleEndian, ReadPodExt};

        let id = reader.read_u32::<LittleEndian>()?;
        let map_id = reader.read_u32::<LittleEndian>()?;
        let x = reader.read_f32::<LittleEndian>()?;
        let y = reader.read_f32::<LittleEndian>()?;
        let z = reader.read_f32::<LittleEndian>()?;
        let radius = reader.read_f32::<LittleEndian>()?;
        let box_x = reader.read_f32::<LittleEndian>()?;
        let box_y = reader.read_f32::<LittleEndian>()?;
        let box_z = reader.read_f32::<LittleEndian>()?;
        let box_o = reader.read_f32::<LittleEndian>()?;

        let shape = if radius > 0.0 {
            AreaTriggerShape::Sphere(radius)
        } else {
            AreaTriggerShape::Box(AreaTriggerBox {
                size_x: box_x,
                size_y: box_y,
                size_z: box_z,
                orientation: box_o,
            })
        };

        Ok(DBCAreaTriggerRow {
            id,
            map_id,
            x,
            y,
            z,
            shape,
        })
    }

    fn get_primary_key(&self) -> Self::PrimaryKeyType {
        self.id
    }
}
