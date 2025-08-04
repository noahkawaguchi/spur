use anyhow::{Context, Result};
use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub frontend_url: String,
    pub bind_addr: String,
    pub jwt_secret: String,
}

impl AppConfig {
    /// Attempts to load the required configuration data from environment variables.
    pub fn load() -> Result<Self> {
        // Expect a .env file in development only
        #[cfg(debug_assertions)]
        dotenvy::dotenv().context("failed to load .env file")?;

        let database_url = env::var("DATABASE_URL").context("failed to load DATABASE_URL")?;
        let frontend_url = env::var("FRONTEND_URL").context("failed to load FRONTEND_URL")?;
        let bind_addr = env::var("BIND_ADDR").context("failed to load BIND_ADDR")?;
        let jwt_secret = env::var("JWT_SECRET").context("failed to load JWT_SECRET")?;

        Ok(Self { database_url, frontend_url, bind_addr, jwt_secret })
    }
}
