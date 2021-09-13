use anyhow::*;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use wrath_auth_db::AuthDatabase;

mod auth;
mod constants;
mod realms;

#[async_std::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let connect_string = &std::env::var("AUTH_DATABASE_URL")?;
    let auth_db = std::sync::Arc::new(AuthDatabase::new(&connect_string).await?);

    task::spawn(realms::receive_realm_pings(auth_db.clone()));

    let tcp_listener = TcpListener::bind("127.0.0.1:3724").await?;
    loop {
        let (stream, _) = tcp_listener.accept().await?;
        task::spawn(handle_incoming_connection(stream, auth_db.clone()));
    }
}

async fn handle_incoming_connection(mut stream: TcpStream, auth_database: std::sync::Arc<AuthDatabase>) -> Result<()> {
    println!("incoming on address {}", stream.local_addr()?.to_string());
    let mut logindata = auth::LoginNumbers::default();

    let mut buf = [0u8; 1024];
    loop {
        let read_len = stream.read(&mut buf).await?;
        if read_len > 0 {
            if buf[0] == 0 {
                logindata = auth::handle_logon_challenge(&mut stream, &buf, &auth_database).await.unwrap();
            } else if buf[0] == 1 {
                auth::handle_logon_proof(&mut stream, &buf, &logindata, &auth_database).await.unwrap();
            } else if buf[0] == 2 {
                println!("reconnect challenge");
            } else if buf[0] == 16 {
                realms::handle_realmlist_request(&mut stream, &logindata, &auth_database).await.unwrap();
            } else {
                println!("unhandled {}", buf[0]);
                return Err(anyhow!("Unhandled command header"));
            }
        } else {
            println!("disconnect");
            stream.shutdown(async_std::net::Shutdown::Both)?;
            break;
        }
    }
    Ok(())
}
