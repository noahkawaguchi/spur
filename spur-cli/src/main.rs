#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod auth;
mod commands;
mod format;
mod friends;
mod input_validators;
mod prompt;
mod request;
mod token_store;

use anyhow::{Context, Result, anyhow};
use auth::AuthCommand;
use clap::Parser;
use commands::{
    Cli,
    Commands::{Add, Check, Friends, Login, Signup},
};
use friends::FriendsCommand;
use prompt::InteractiveAuthPrompt;
use request::ApiRequestClient;
use std::env;
use token_store::{LocalTokenStore, TokenStore};
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

    let client = ApiRequestClient::new(backend_url)?;
    let store = LocalTokenStore::new(&home_dir)?;

    let command = Cli::parse().command;

    // Auth commands don't require the user to already have a token, while all others do
    let result = if matches!(command, Signup | Login | Check) {
        let auth = AuthCommand { prompt: InteractiveAuthPrompt, store, client };

        match command {
            Signup => auth.signup().await,
            Login => auth.login().await,
            Check => auth.check().await,
            _ => unreachable!(),
        }
    } else {
        let token = &store.load()?;
        let friends = FriendsCommand { client, token };

        match command {
            Signup | Login | Check => unreachable!(),
            Add { username } => friends.add_friend(username).await,
            Friends { pending } => friends.list_friends(pending).await,
        }
    };

    println!("{}", format::color_first_line(result)?);
    Ok(())
}
