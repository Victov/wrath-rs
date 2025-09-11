use std::time::Duration;

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::AsyncComputeTaskPool};
use wrath_auth_db::AuthDatabase;
use wrath_realm_db::RealmDatabase;

#[derive(Resource)]
pub struct AuthDatabaseResource(pub AuthDatabase);

#[derive(Resource)]
pub struct RealmDatabaseResource(pub RealmDatabase);

/// Bevy system that boots up the auth database connection.
/// The initial database connection is asynchronous, but since we're still in the server startup and we don't have to worry
/// about keeping the framerate stable, we can simple block and wait on it.
pub(super) fn setup_auth_database(mut commands: Commands) -> Result {
    // Read environment variable for connection timeout
    let db_connect_timeout = Duration::from_secs(std::env::var("DB_CONNECT_TIMEOUT_SECONDS")?.parse()?);

    // Spawn an asynchronous task that may span several frames in order to connect to the authentication database
    if let Ok(mut connection_result) = bevy::tasks::block_on(async move {
        // Connect (may take time asynchronously)
        let auth_database = AuthDatabase::new(&std::env::var("AUTH_DATABASE_URL")?, db_connect_timeout).await?;

        // When connected, push a command to the commandqueue to insert it as a resource.
        let mut command_queue = CommandQueue::default();
        command_queue.push(move |world: &mut bevy::prelude::World| world.insert_resource(AuthDatabaseResource(auth_database)));

        info!("Auth database connected");

        Result::<CommandQueue, BevyError>::Ok(command_queue)
    }) {
        commands.append(&mut connection_result);
    }

    Ok(())
}

/// Bevy system that boots up the realm database connection.
/// Pretty much identical to `setup_auth_database`, so for more information check there.
pub(super) fn setup_realm_database(mut commands: Commands) -> Result {
    let db_connect_timeout = Duration::from_secs(std::env::var("DB_CONNECT_TIMEOUT_SECONDS")?.parse()?);
    if let Ok(mut connection_result) = bevy::tasks::block_on(async move {
        let realm_database = RealmDatabase::new(&std::env::var("REALM_DATABASE_URL")?, db_connect_timeout).await?;
        let mut command_queue = CommandQueue::default();
        command_queue.push(move |world: &mut bevy::prelude::World| world.insert_resource(RealmDatabaseResource(realm_database)));

        info!("Realm database connected");

        Result::<CommandQueue, BevyError>::Ok(command_queue)
    }) {
        commands.append(&mut connection_result);
    }

    Ok(())
}
