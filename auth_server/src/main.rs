use core::sync::atomic::{AtomicBool, Ordering};

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
    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let ctrlc_handler = async_ctrlc::CtrlC::new().expect("Failed to setup ctrl+c handler");
    task::spawn(async move {
        ctrlc_handler.await;
        r.store(false, Ordering::Relaxed);
        println!("Received a ctrl+C. Performing graceful shutdown");
    });

    dotenv::dotenv().ok();
    let connect_string = &std::env::var("AUTH_DATABASE_URL")?;
    let auth_db = std::sync::Arc::new(AuthDatabase::new(&connect_string).await?);

    task::spawn(realms::receive_realm_pings(auth_db.clone()));

    let tcp_listener = TcpListener::bind("127.0.0.1:3724").await?;

    let loop_accept_duration = std::time::Duration::from_secs_f32(1.0f32);
    while running.load(Ordering::Relaxed) {
        let accept_future = tcp_listener.accept();
        if let Ok(Ok((stream, _))) = async_std::future::timeout(loop_accept_duration, accept_future).await {
            task::spawn(handle_incoming_connection(stream, auth_db.clone()));
        }
    }

    Ok(())
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
