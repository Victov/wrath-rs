use bevy::prelude::*;

pub(super) struct StartupProcedurePlugin;

impl Plugin for StartupProcedurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (super::databases::setup_auth_database, super::databases::setup_realm_database));
    }
}
