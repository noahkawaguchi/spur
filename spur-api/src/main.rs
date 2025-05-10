mod models;

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use dotenvy::dotenv;
use models::user::NewUser;
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

    let alice = NewUser {
        name: "Alice",
        email: "alice@example.com",
        username: "alice123",
        password_hash: "xc414hb14bs",
    };

    models::user::insert_new(&pool, &alice).await?;
    let got_alice = models::user::get_by_email(&pool, alice.email).await?;

    println!("{got_alice:?}");

    let app = Router::new().route("/", get(hello_world));
    let listener = TcpListener::bind(&backend_addr).await?;

    println!("Listening on http://{}...", &backend_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello_world() -> &'static str { "Hello, World!" }
