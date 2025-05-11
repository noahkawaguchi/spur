mod handlers;
mod models;
mod repositories;
mod services;

use anyhow::{Context, Result};
use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use handlers::signup;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    let database_url = env::var("DATABASE_URL").context("failed to load DATABASE_URL")?;
    let backend_addr = env::var("BACKEND_ADDR").context("failed to load BACKEND_ADDR")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/signup", post(signup))
        .with_state(pool);

    let listener = TcpListener::bind(&backend_addr).await?;

    println!("Listening on http://{}...", &backend_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello_world() -> &'static str { "Hello, World!" }
