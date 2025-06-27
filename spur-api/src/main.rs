#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod domain;
mod handler;
mod middleware;
mod models;
mod repository;
mod router;
mod service;
mod state;
mod technical_error;
mod utils;

#[cfg(test)]
mod test_utils;

use anyhow::Result;
use config::AppConfig;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    let state = AppState::build(pool, config.jwt_secret);
    let app = router::create(state);
    let listener = TcpListener::bind(&config.bind_addr).await?;

    #[cfg(debug_assertions)]
    println!("Listening on http://{}...", &config.bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
