#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod handlers;
mod models;
mod repositories;
mod services;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use config::{AppConfig, AppState};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let state = AppState { pool, jwt_secret: config.jwt_secret };

    let app = Router::new()
        .route("/signup", post(handlers::signup))
        .route("/login", post(handlers::login))
        .route("/check", get(handlers::check))
        .with_state(state);

    let listener = TcpListener::bind(&config.bind_addr).await?;

    if cfg!(debug_assertions) {
        println!("Listening on http://{}...", &config.bind_addr);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
