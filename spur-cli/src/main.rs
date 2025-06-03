#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod auth;
mod commands;
mod content;
mod format;
mod friends;
mod input_validators;
mod interactive_auth;
mod request;
mod token_store;

use anyhow::{Context, Result, anyhow};
use auth::AuthCommand;
use clap::Parser;
use commands::{Cli, Cmd};
use content::ContentCommand;
use friends::FriendsCommand;
use interactive_auth::InteractiveAuthPrompt;
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
    let result = if matches!(command, Cmd::Signup | Cmd::Login | Cmd::Check) {
        let auth = AuthCommand { prompt: InteractiveAuthPrompt, store, client };

        match command {
            Cmd::Signup => auth.signup().await,
            Cmd::Login => auth.login().await,
            Cmd::Check => auth.check().await,
            _ => unreachable!(),
        }
    } else {
        let token = &store.load()?;
        let friends = FriendsCommand { client: client.clone(), token };
        let content = ContentCommand { client, token };

        match command {
            // Auth commands handled above
            Cmd::Signup | Cmd::Login | Cmd::Check => unreachable!(),

            // Friendship commands
            Cmd::Add { username } => friends.add_friend(username).await,
            Cmd::Friends => friends.list_friends(false).await,
            Cmd::Requests => friends.list_friends(true).await,

            // Prompt and post commands
            Cmd::Prompt { body } => content.new_prompt(body).await,
            Cmd::Write { prompt_id } => content.write_post(prompt_id).await,
            Cmd::Read { post_id } => content.read_post(post_id).await,
            Cmd::Profile { username } => content.user_content(Some(username)).await,
            Cmd::Me => content.user_content(None).await,
            Cmd::Feed => content.feed().await,
        }
    };

    println!("{}", format::color_first_line(result)?);
    Ok(())
}
