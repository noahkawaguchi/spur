#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod auth;
mod commands;
mod error_response;
mod input_validators;
mod prompt;
mod token_store;

use anyhow::{Context, Result, anyhow};
use auth::AuthCmd;
use clap::Parser;
use colored::Colorize;
use commands::{
    Cli,
    Commands::{Check, Login, Signup},
};
use prompt::InteractiveAuthPrompt;
use std::env;
use token_store::LocalTokenStore;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // Expect a .env file in development only
    if cfg!(debug_assertions) {
        dotenvy::dotenv()?;
    }

    let backend_url_string = env::var("BACKEND_URL").context("failed to load BACKEND_URL")?;
    let backend_url = Url::parse(&backend_url_string).context("failed to parse BACKEND_URL")?;

    let home_dir = dirs_next::home_dir().ok_or_else(|| anyhow!("could not find home directory"))?;
    let auth = AuthCmd { prompt: InteractiveAuthPrompt, store: LocalTokenStore::new(&home_dir)? };

    let result = match Cli::parse().command {
        Signup => auth.signup(&backend_url).await,
        Login => auth.login(&backend_url).await,
        Check => auth.check(&backend_url).await,
    };

    match result {
        Err(e) => Err(anyhow!(e.to_string().red())),
        Ok(msg) => {
            println!("{}", msg.green());
            Ok(())
        }
    }
}
