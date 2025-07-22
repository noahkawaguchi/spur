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
use axum::http::{HeaderValue, Method, header};
use config::AppConfig;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    let state = AppState::build(pool, config.jwt_secret);
    let app = router::create(state).layer(cors);
    let listener = TcpListener::bind(&config.bind_addr).await?;

    #[cfg(debug_assertions)]
    println!("Listening on http://{}...", &config.bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
