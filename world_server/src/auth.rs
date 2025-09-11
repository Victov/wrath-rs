use std::time::Duration;

use crate::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use podio::{BigEndian, WritePodExt};
use smol::net::UdpSocket;

/*

Pre-bevy style authentication server heartbeats. Kept for reference during bevy refactor. To be cleaned up
once everything is back in a functional state.

pub async fn auth_server_heartbeats() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    socket.connect("127.0.0.1:1234").await?;
    let num_players_online = 10u32;

    info!("My realm ID = {}", std::env::var("REALM_ID")?);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let buf = Vec::<u8>::new();
        let mut writer = std::io::Cursor::new(buf);
        writer.write_u8(0u8)?; //HEARTBEAT
        writer.write_u8(std::env::var("REALM_ID")?.parse()?)?; //Realm ID
        writer.write_u32::<BigEndian>(num_players_online)?;

        socket.send(&writer.into_inner()).await?;
    }
}
*/

pub(super) struct AuthServerHeartbeatPlugin;

impl Plugin for AuthServerHeartbeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_auth_server_heartbeat);
        app.add_systems(Update, auth_server_heartbeat);
    }
}

#[derive(Component)]
struct AuthServerHeartbeat {
    timer: bevy::time::Timer,
    udp_socket: UdpSocket,
}

#[derive(Resource, Default)]
struct AuthServerHeartbeatData {
    num_players_online: u32,
}

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

fn auth_server_heartbeat(
    mut q: Query<(Entity, &mut AuthServerHeartbeat)>,
    time: Res<Time>,
    heartbeat_data: Res<AuthServerHeartbeatData>,
) -> bevy::prelude::Result {
    for (_, mut heartbeat) in q.single_mut() {
        // Progress the timer
        heartbeat.timer.tick(time.delta());

        let cloned_socket = heartbeat.udp_socket.clone();
        let num_players_online = heartbeat_data.num_players_online;
        // If it has finished, send a heartbeat.
        if heartbeat.timer.finished() {
            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let buf: [u8; 6] = [0; 6];
                    let mut writer = std::io::Cursor::new(buf);
                    writer.write_u8(0u8)?; //HEARTBEAT
                    writer.write_u8(std::env::var("REALM_ID")?.parse()?)?; //Realm ID
                    writer.write_u32::<BigEndian>(num_players_online)?;

                    cloned_socket.send(&writer.into_inner()).await?;
                    info!("sending heartbeat!");
                    bevy::prelude::Result::<(), BevyError>::Ok(())
                })
                .detach();
        }
    }
    Ok(())
}
