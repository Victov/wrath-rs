use anyhow::Result;
use async_std::task;
use cmdparse::{parse, Parsable};
use std::io::{self, BufRead};
use std::sync::{atomic::AtomicBool, Arc};
use tracing::{info, warn};

#[derive(Debug, PartialEq, Eq, Parsable)]
enum WrathRealmConsoleCommand {
    Exit,
}

pub async fn process_console_commands(running_bool: Arc<AtomicBool>) -> Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(string) => {
                let cmd = parse::<_, WrathRealmConsoleCommand>(&string, ());
                match cmd {
                    Ok(parsed_cmd) => {
                        task::spawn(handle_command(parsed_cmd, running_bool.clone()));
                    }
                    Err(e) => warn!("Could not parse command. {}", e),
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}

async fn handle_command(cmd: WrathRealmConsoleCommand, running_bool: Arc<AtomicBool>) -> Result<()> {
    let result = match cmd {
        WrathRealmConsoleCommand::Exit => handle_exit(running_bool).await,
    };

    if let Err(e) = result {
        warn!("Error: {}", e);
    }
    Ok(())
}

async fn handle_exit(running_bool: Arc<AtomicBool>) -> Result<()> {
    info!("Starting graceful shutdown from exit command");
    running_bool.store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
