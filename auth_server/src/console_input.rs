use anyhow::Result;
use async_std::task;
use cmdparse::{parse, Parsable};
use std::io::{self, BufRead};
use tracing::{info, warn};
use wow_srp::{normalized_string::NormalizedString, server::SrpVerifier};
use wrath_auth_db::AuthDatabase;

#[derive(Debug, PartialEq, Eq, Parsable)]
enum WrathConsoleCommand {
    CreateAccount(String, String),
    Ban(String),
    Unban(String),
}

pub async fn process_console_commands(auth_db: std::sync::Arc<AuthDatabase>) -> Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(string) => {
                let cmd = parse::<_, WrathConsoleCommand>(&string, ());
                match cmd {
                    Ok(parsed_cmd) => {
                        task::spawn(handle_command(parsed_cmd, auth_db.clone()));
                    }
                    Err(e) => warn!("Could not parse command. {}", e),
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}

async fn handle_command(cmd: WrathConsoleCommand, auth_db: std::sync::Arc<AuthDatabase>) -> Result<()> {
    match cmd {
        WrathConsoleCommand::CreateAccount(username, password) => handle_create_account(&username, &password, &auth_db).await?,
        WrathConsoleCommand::Ban(username) => handle_ban(&username, &auth_db).await?,
        WrathConsoleCommand::Unban(username) => handle_unban(&username, &auth_db).await?,
    }
    Ok(())
}

async fn handle_create_account(username: &str, password: &str, auth_db: &std::sync::Arc<AuthDatabase>) -> Result<()> {
    let u_normalised = NormalizedString::new(username)?;
    let p_normalised = NormalizedString::new(password)?;
    let v = SrpVerifier::from_username_and_password(u_normalised, p_normalised);

    auth_db
        .create_account(v.username(), &hex::encode(v.password_verifier()), &hex::encode(v.salt()))
        .await?;

    info!("Account {} created", username);
    Ok(())
}

async fn handle_ban(username: &str, auth_db: &std::sync::Arc<AuthDatabase>) -> Result<()> {
    auth_db.set_account_ban_status(username, true).await?;
    info!("Account {} banned", username);
    Ok(())
}

async fn handle_unban(username: &str, auth_db: &std::sync::Arc<AuthDatabase>) -> Result<()> {
    auth_db.set_account_ban_status(username, false).await?;
    info!("Account {} unbanned", username);
    Ok(())
}
