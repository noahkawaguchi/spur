#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod domain;
mod handler;
mod models;
mod repository;
mod service;
mod technical_error;

#[cfg(test)]
mod test_utils;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use config::{AppConfig, AppState};
use domain::user::UserManager;
use handler::{auth, friendship};
use repository::{friendship::FriendshipRepo, user::UserRepo};
use service::{friendship::FriendshipSvc, user::UserSvc};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    let user_svc = Arc::new(UserSvc::new(UserRepo::new(pool.clone()))) as Arc<dyn UserManager>;

    let friendship_svc = Arc::new(FriendshipSvc::new(
        FriendshipRepo::new(pool),
        Arc::clone(&user_svc),
    ));

    let state = AppState { jwt_secret: config.jwt_secret, user_svc, friendship_svc };

    let app = Router::new()
        .route("/signup", post(auth::signup))
        .route("/login", post(auth::login))
        .route("/check", get(auth::check))
        .route("/add", post(friendship::add_friend))
        .route("/friends", get(friendship::get_friends))
        .route("/requests", get(friendship::get_requests))
        .with_state(state);

    let listener = TcpListener::bind(&config.bind_addr).await?;

    if cfg!(debug_assertions) {
        println!("Listening on http://{}...", &config.bind_addr);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
