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
use domain::{friendship::service::FriendshipManager, user::UserManager};
use handler::{auth, friendship, prompt};
use repository::{friendship::FriendshipRepo, prompt::PromptRepo, user::UserRepo};
use service::{friendship::FriendshipSvc, prompt::ContentSvc, user::UserSvc};
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
        FriendshipRepo::new(pool.clone()),
        Arc::clone(&user_svc),
    )) as Arc<dyn FriendshipManager>;

    let prompt_svc = Arc::new(ContentSvc::new(
        PromptRepo::new(pool),
        Arc::clone(&friendship_svc),
        Arc::clone(&user_svc),
    ));

    let state = AppState { jwt_secret: config.jwt_secret, user_svc, friendship_svc, prompt_svc };

    let app = Router::new()
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .route("/auth/check", get(auth::check))
        .route("/friends", post(friendship::add_friend))
        .route("/friends", get(friendship::get_friends))
        .route("/prompts", post(prompt::new_prompt))
        .route("/prompts/{prompt_id}", get(prompt::get_for_writing))
        .route("/prompts", get(prompt::get_by_author))
        .route("/prompts/friends", get(prompt::all_friend_prompts))
        .with_state(state);

    let listener = TcpListener::bind(&config.bind_addr).await?;

    if cfg!(debug_assertions) {
        println!("Listening on http://{}...", &config.bind_addr);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
