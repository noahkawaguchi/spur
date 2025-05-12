#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod auth;
mod commands;
mod error_response;
mod input_validators;

use anyhow::{Context, Result};
use clap::Parser;
use commands::{
    Cli,
    Commands::{Check, Login, Signup},
};
use std::env;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let backend_url = env::var("BACKEND_URL").context("failed to load BACKEND_URL")?;
    let backend_url = Url::parse(&backend_url).context("failed to parse BACKEND_URL")?;

    match Cli::parse().command {
        Signup => auth::signup(&backend_url).await?,
        Login => auth::login(&backend_url).await?,
        Check => println!("check used!"),
    }

    Ok(())
}
