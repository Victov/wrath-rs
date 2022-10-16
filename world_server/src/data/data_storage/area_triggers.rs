use std::sync::Arc;

use wow_dbc::{
    wrath_tables::{area_trigger::AreaTriggerKey, map::MapKey},
    DbcTable,
};
use wrath_realm_db::{areatrigger_teleport::DBAreaTriggerTeleport, RealmDatabase};

use crate::prelude::*;

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
pub enum AreaTriggerPurpose {
    Teleport(DBAreaTriggerTeleport),
    RestedArea,
    Unknown,
}

#[derive(Debug)]
pub struct AreaTrigger {
    pub id: AreaTriggerKey,
    pub map_id: MapKey,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub shape: AreaTriggerShape,
    pub purpose: AreaTriggerPurpose,
}

impl super::DataStorage {
    pub(super) async fn load_area_triggers(&mut self, dbc_path: impl Into<&str>, realm_db: Arc<RealmDatabase>) -> Result<()> {
        let mut area_triggers_local: Option<wow_dbc::wrath_tables::area_trigger::AreaTrigger> = None;
        super::load_standard_dbc(dbc_path, &mut area_triggers_local).await?;

        if let Some(area_triggers_local) = area_triggers_local {
            for areatrigger in area_triggers_local.rows().iter() {
                let shape = if areatrigger.radius > 0.0 {
                    AreaTriggerShape::Sphere(areatrigger.radius)
                } else {
                    AreaTriggerShape::Box(AreaTriggerBox {
                        size_x: areatrigger.box_length,
                        size_y: areatrigger.box_width,
                        size_z: areatrigger.box_height,
                        orientation: areatrigger.box_yaw,
                    })
                };

                let purpose = if let Ok(teleport_data) = realm_db.get_areatrigger_teleport(areatrigger.id.id as u32).await {
                    AreaTriggerPurpose::Teleport(teleport_data)
                } else if let Ok(_rested_area_data) = realm_db.get_areatrigger_rested_zone(areatrigger.id.id as u32).await {
                    AreaTriggerPurpose::RestedArea
                } else {
                    AreaTriggerPurpose::Unknown
                };

                let areatrigger_final = AreaTrigger {
                    id: areatrigger.id,
                    map_id: areatrigger.continent_id,
                    x: areatrigger.pos[0],
                    y: areatrigger.pos[1],
                    z: areatrigger.pos[2],
                    shape,
                    purpose,
                };

                self.area_triggers.insert(areatrigger_final.id.id as u32, areatrigger_final);
            }
        }

        Ok(())
    }
}
