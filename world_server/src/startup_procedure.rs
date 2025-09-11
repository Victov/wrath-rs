use crate::data::DataStorage;
use bevy::{prelude::*, tasks};

pub(super) struct StartupProcedurePlugin;

impl Plugin for StartupProcedurePlugin {
    /// When starting, first connect to the required databases. Systems given to the [Pre|Post]Startup schedules are only run once by bevy.
    /// It is important that database connections are in place before core Startup, as other initialization may depend on reading data.
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            (super::databases::setup_auth_database, super::databases::setup_realm_database),
        );
        app.add_systems(Startup, setup_data_storage);
    }
}

/// During a transition phase into Bevy, we wrap the DataStorage in this struct in order to keep changes minimal.
/// Eventually DataStorage itself may be turned into a proper Bevy Resource, eliminating the need for this wrapper.
#[derive(Resource)]
pub struct DataStorageResource(pub DataStorage);

/// Load DBC files. This is an asynchronous process, leaving possibilities for parallelism during startup, but for now we simply block.
fn setup_data_storage(mut commands: Commands, realm_database: Res<super::databases::RealmDatabaseResource>) -> Result {
    tasks::block_on(async move {
        let mut data_storage = super::data::DataStorage::default();
        data_storage.load(&realm_database.0).await?;
        commands.insert_resource(DataStorageResource(data_storage));
        Ok(())
    })
}
