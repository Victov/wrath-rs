use std::time::Duration;

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::AsyncComputeTaskPool};
use wrath_auth_db::AuthDatabase;
use wrath_realm_db::RealmDatabase;

/// Wrapper for the auth database, in order to be able to insert it into the Bevy ECS as a resource.
/// Possibly eventually this wrapper can be resolved and the AuthDatabase can be turned into a bevy resource directly
/// but this is postponed in order to keep changes minimal during an already monstrously big refactor.
#[derive(Resource)]
pub struct AuthDatabaseResource(pub AuthDatabase);

/// See `AuthDatabaseResource` for reasoning why this wrapper exists.
#[derive(Resource)]
pub struct RealmDatabaseResource(pub RealmDatabase);

/// Bevy system that boots up the auth database connection.
/// The initial database connection is asynchronous, but since we're still in the server startup and we don't have to worry
/// about keeping the framerate stable, we can simple block and wait on it.
pub(super) fn setup_auth_database(mut commands: Commands) -> Result {
    bevy::tasks::block_on(async move {
        let db_connect_timeout = Duration::from_secs(std::env::var("DB_CONNECT_TIMEOUT_SECONDS")?.parse()?);
        let auth_database = AuthDatabase::new(&std::env::var("AUTH_DATABASE_URL")?, db_connect_timeout).await?;

        commands.insert_resource(AuthDatabaseResource(auth_database));

        info!("Auth database connected");
        Ok(())
    })
}

/// Bevy system that boots up the realm database connection.
/// Pretty much identical to `setup_auth_database`, so for more information check there.
pub(super) fn setup_realm_database(mut commands: Commands) -> Result {
    bevy::tasks::block_on(async move {
        let db_connect_timeout = Duration::from_secs(std::env::var("DB_CONNECT_TIMEOUT_SECONDS")?.parse()?);
        let realm_database = RealmDatabase::new(&std::env::var("REALM_DATABASE_URL")?, db_connect_timeout).await?;
        commands.insert_resource(RealmDatabaseResource(realm_database));

        info!("Realm database connected");
        Ok(())
    })
}
