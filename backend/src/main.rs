#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod api;
mod app_services;
mod config;
mod domain;
mod infra;
mod map_into;
mod models;
mod read_models;
mod state;

#[cfg(test)]
mod test_utils;

use crate::api::router;
use anyhow::Result;
use config::AppConfig;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    let state = AppState::init(&config).await?;
    let app = router::build(state, &config.frontend_url)?;
    let listener = TcpListener::bind(&config.bind_addr).await?;

    #[cfg(debug_assertions)]
    println!("Listening on http://{} ...", &config.bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
