use crate::prelude::*;
use smol::net::UdpSocket;
use std::time::Duration;

pub(super) struct AuthServerHeartbeatPlugin;

/// During startup, a UDP socket is created to send heartbeats.
/// During normal operation, a bevy timer is kept in order to keep track of heartbeat times and a detached asynchronous task is used to send the heartbeat.
impl Plugin for AuthServerHeartbeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_auth_server_heartbeat);
        app.add_systems(Update, auth_server_heartbeat);
    }
}

/// Only one entity with this component should exist, created during `setup_auth_server_heartbeat`.
/// Contents of this entity should remain hidden to the rest of the application. If you are meaning to modify heartbeat data
/// see `AuthServerHeartbeatData`.
#[derive(Component)]
struct AuthServerHeartbeat {
    timer: bevy::time::Timer,
    udp_socket: UdpSocket,
}

/// Public data to be modified by other running parts of the server. The contents of this are sent to the auth server.
#[derive(Resource, Default)]
struct AuthServerHeartbeatData {
    num_players_online: u32,
}

/// One-time initialization during server startup.
fn setup_auth_server_heartbeat(mut commands: Commands) -> bevy::prelude::Result {
    bevy::tasks::block_on(async move {
        commands.insert_resource(AuthServerHeartbeatData::default());

        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        socket.connect("127.0.0.1:1234").await?;

        commands.spawn_empty().insert(AuthServerHeartbeat {
            timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
            udp_socket: socket,
        });

        Ok(())
    })
}

/// Happens every server tick.
fn auth_server_heartbeat(
    mut q: Query<(Entity, &mut AuthServerHeartbeat)>,
    time: Res<Time>,
    heartbeat_data: Res<AuthServerHeartbeatData>,
) -> bevy::prelude::Result {
    use podio::{BigEndian, WritePodExt};

    if let Ok((_, mut heartbeat)) = q.single_mut() {
        // Progress the timer
        heartbeat.timer.tick(time.delta());

        let cloned_socket = heartbeat.udp_socket.clone();
        let num_players_online = heartbeat_data.num_players_online;

        // If it has finished, send a heartbeat. This is a repeating timer.
        if heartbeat.timer.finished() {
            bevy::tasks::AsyncComputeTaskPool::get()
                .spawn(async move {
                    let buf: [u8; 6] = [0; 6];
                    let mut writer = std::io::Cursor::new(buf);
                    writer.write_u8(0u8)?; //HEARTBEAT
                    writer.write_u8(std::env::var("REALM_ID")?.parse()?)?; //Realm ID
                    writer.write_u32::<BigEndian>(num_players_online)?;

                    cloned_socket.send(&writer.into_inner()).await?;

                    bevy::prelude::Result::<(), BevyError>::Ok(())
                })
                .detach();
        }
    }
    Ok(())
}
