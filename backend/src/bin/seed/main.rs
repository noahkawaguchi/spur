#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod friendship;
mod time_utils;
mod user;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    user::seed(&pool).await?; // Users must be seeded first
    friendship::seed(&pool).await?;

    Ok(())
}
