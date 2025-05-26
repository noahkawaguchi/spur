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
use handler::{auth, friendship};
use repository::{friendship::FriendshipRepo, user::UserRepo};
use service::{auth::AuthSvc, friendship::FriendshipSvc};
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

    let user_repo = UserRepo::new_arc(pool.clone());
    let friendship_repo = FriendshipRepo::new_arc(pool);

    let auth_svc = AuthSvc::new(Arc::clone(&user_repo));
    let friendship_svc = FriendshipSvc::new(friendship_repo, user_repo);

    let state = AppState {
        jwt_secret: config.jwt_secret,
        auth_svc: Arc::new(auth_svc),
        friendship_svc: Arc::new(friendship_svc),
    };

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
