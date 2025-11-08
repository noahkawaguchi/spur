#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod friendship;
mod post;
mod time_utils;
mod user;

use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};

/// Connects to the database specified in the environment variable `DATABASE_URL` and inserts seed
/// data if the tables are empty. Assumes that migrations have already been applied.
#[tokio::main]
async fn main() -> Result<()> {
    spur::logger::init_with_default(log::LevelFilter::Info);
    log::info!("Seed binary starting...");

    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    log::info!("Connected to database");

    // Only attempt to insert seed data if the tables are empty
    if rows_exist(&pool).await? {
        log::warn!("Existing row(s) found in the database, skipping seeding");
    } else {
        user::seed(&pool).await?; // Users must be seeded first
        friendship::seed(&pool).await?;
        post::seed(&pool).await?;
    }

    Ok(())
}

/// Determines whether any rows exist in the users, friendship, or post tables.
async fn rows_exist(pool: &PgPool) -> Result<bool> {
    sqlx::query_scalar!(
        "
        SELECT EXISTS (
            SELECT 1 FROM users
            UNION ALL
            SELECT 1 FROM friendship
            UNION ALL
            SELECT 1 FROM post
            LIMIT 1
        )
        "
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
    .and_then(|maybe_row| maybe_row.context("unexpected NULL result from SELECT EXISTS"))
}
