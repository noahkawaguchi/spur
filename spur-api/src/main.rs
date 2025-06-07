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
mod utils;

#[cfg(test)]
mod test_utils;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use config::{AppConfig, AppState};
use domain::{
    content::service::{PostManager, PromptManager},
    friendship::service::FriendshipManager,
    user::UserManager,
};
use handler::{auth, content, friendship, post, prompt};
use repository::{friendship::FriendshipRepo, post::PostRepo, prompt::PromptRepo, user::UserRepo};
use service::{
    content::ContentSvc, friendship::FriendshipSvc, post::PostSvc, prompt::PromptSvc, user::UserSvc,
};
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

    let prompt_svc = Arc::new(PromptSvc::new(
        PromptRepo::new(pool.clone()),
        Arc::clone(&friendship_svc),
    )) as Arc<dyn PromptManager>;

    let post_svc = Arc::new(PostSvc::new(
        PostRepo::new(pool.clone()),
        Arc::clone(&friendship_svc),
        Arc::clone(&prompt_svc),
    )) as Arc<dyn PostManager>;

    let content_svc = Arc::new(ContentSvc::new(
        Arc::clone(&user_svc),
        Arc::clone(&friendship_svc),
        Arc::clone(&prompt_svc),
        Arc::clone(&post_svc),
    ));

    let state = AppState {
        jwt_secret: config.jwt_secret,
        user_svc,
        friendship_svc,
        prompt_svc,
        post_svc,
        content_svc,
    };

    let app = Router::new()
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .route("/auth/check", get(auth::check))
        .route("/friends", post(friendship::add_friend))
        .route("/friends", get(friendship::get_friends))
        .route("/prompts", post(prompt::create_new))
        .route("/prompts/{prompt_id}", get(prompt::get_for_writing))
        .route("/posts", post(post::create_new))
        .route("/posts/{post_id}", get(post::get_for_reading))
        .route("/content", get(content::user_content))
        .route("/content/friends", get(content::friends_content))
        .with_state(state);

    let listener = TcpListener::bind(&config.bind_addr).await?;

    if cfg!(debug_assertions) {
        println!("Listening on http://{}...", &config.bind_addr);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
