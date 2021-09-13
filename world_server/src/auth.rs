use anyhow::Result;
use async_std::net::UdpSocket;
use podio::{BigEndian, WritePodExt};

pub async fn auth_server_heartbeats() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    socket.connect("127.0.0.1:1234").await?;
    let num_players_online = 10u32;

    println!("REALM_ID = {}", std::env::var("REALM_ID")?);
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
