#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use anyhow::Result;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};

/// Database migrator that embeds migrations into the binary at compile time.
static MIGRATOR: Migrator = sqlx::migrate!();

/// Connects to the database specified in the environment variable `DATABASE_URL` and validates the
/// migrations embedded into the binary, applying any that are pending.
#[tokio::main]
async fn main() -> Result<()> {
    spur::logger::init_with_default(log::LevelFilter::Info);
    log::info!("Migrate binary starting...");

    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    log::info!("Connected to database");

    MIGRATOR.run(&pool).await?;
    log::info!("Migrations applied/validated");

    Ok(())
}
