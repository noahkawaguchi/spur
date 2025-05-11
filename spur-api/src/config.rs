use anyhow::{Context, Result};
use axum::extract::FromRef;
use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub backend_addr: String,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv()?;

        let database_url = env::var("DATABASE_URL").context("failed to load DATABASE_URL")?;
        let backend_addr = env::var("BACKEND_ADDR").context("failed to load BACKEND_ADDR")?;
        let jwt_secret = env::var("JWT_SECRET").context("failed to load JWT_SECRET")?;

        Ok(Self { database_url, backend_addr, jwt_secret })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub jwt_secret: String,
}

pub struct DbConfig {
    pub pool: sqlx::PgPool,
}

impl FromRef<AppState> for DbConfig {
    fn from_ref(input: &AppState) -> Self { Self { pool: input.pool.clone() } }
}

pub struct JwtConfig {
    pub secret: String,
}

impl FromRef<AppState> for JwtConfig {
    fn from_ref(input: &AppState) -> Self { Self { secret: input.jwt_secret.clone() } }
}
