use std::sync::Arc;

use dbc::AreaTriggerShape;
use wrath_realm_db::{areatrigger_teleport::DBAreaTriggerTeleport, RealmDatabase};

use crate::prelude::*;

#[derive(Debug)]
pub enum AreaTriggerPurpose {
    Teleport(DBAreaTriggerTeleport),
    RestedArea,
    Unknown,
}

#[derive(Debug)]
pub struct AreaTrigger {
    pub id: u32,
    pub map_id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub shape: AreaTriggerShape,
    pub purpose: AreaTriggerPurpose,
}

impl super::DataStorage {
    pub(super) async fn load_area_triggers(&mut self, dbc_storage: &mut dbc::DBCStorage, realm_db: Arc<RealmDatabase>) -> Result<()> {
        dbc_storage.load_dbc_area_triggers().await?;

        let dbc_triggers = dbc_storage.get_dbc_area_triggers()?;
        for areatrigger in dbc_triggers.iter() {
            let mut areatrigger_final = AreaTrigger {
                id: areatrigger.id,
                map_id: areatrigger.map_id,
                x: areatrigger.x,
                y: areatrigger.y,
                z: areatrigger.z,
                shape: areatrigger.shape,
                purpose: AreaTriggerPurpose::Unknown,
            };

            if let Ok(teleport_data) = realm_db.get_areatrigger_teleport(areatrigger.id).await {
                areatrigger_final.purpose = AreaTriggerPurpose::Teleport(teleport_data);
            } else if let Ok(_rested_area_data) = realm_db.get_areatrigger_rested_zone(areatrigger.id).await {
                areatrigger_final.purpose = AreaTriggerPurpose::RestedArea;
            }

            self.area_triggers.insert(areatrigger_final.id, areatrigger_final);
        }

        dbc_storage.unload_dbc_area_triggers();
        Ok(())
    }
}
