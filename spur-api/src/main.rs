#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod handlers;
mod models;
mod repositories;
mod services;

#[cfg(test)]
mod test_utils;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use config::{AppConfig, AppState};
use handlers::auth_handlers;
use repositories::user_repo::UserRepo;
use services::auth_svc::AuthSvc;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let user_repo = Arc::new(UserRepo::new(pool));
    let auth_svc = AuthSvc::new(user_repo);

    let state = AppState { auth_svc: Arc::new(auth_svc), jwt_secret: config.jwt_secret };

    let app = Router::new()
        .route("/signup", post(auth_handlers::signup))
        .route("/login", post(auth_handlers::login))
        .route("/check", get(auth_handlers::check))
        .with_state(state);

    let listener = TcpListener::bind(&config.bind_addr).await?;

    if cfg!(debug_assertions) {
        println!("Listening on http://{}...", &config.bind_addr);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
