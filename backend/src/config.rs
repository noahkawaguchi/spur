use anyhow::{Context, Result, anyhow};
use std::{
    any::type_name,
    env::{self, VarError},
    fmt::Display,
    str::FromStr,
};

pub struct AppConfig {
    pub database_url: String,
    pub frontend_url: String,
    pub bind_addr: String,
    pub jwt_secret: String,
    pub max_pool_connections: u32,
    pub db_conn_timeout_secs: u64,
}

impl AppConfig {
    /// Attempts to load the required configuration data from environment variables.
    pub fn load() -> Result<Self> {
        // Expect a .env file in development only
        #[cfg(debug_assertions)]
        dotenvy::dotenv().context("failed to load .env file")?;

        Ok(Self {
            // Definitely different in dev and prod?
            // -> No defaults, must be set as environment variables
            database_url: Self::get_env("DATABASE_URL")?,
            frontend_url: Self::get_env("FRONTEND_URL")?,
            bind_addr: Self::get_env("BIND_ADDR")?,
            jwt_secret: Self::get_env("JWT_SECRET")?,

            // Possibly the same in dev and prod?
            // -> Hardcoded defaults that can be overridden with environment variables
            max_pool_connections: Self::get_env_or_default(10, "MAX_POOL_CONNECTIONS")?,
            db_conn_timeout_secs: Self::get_env_or_default(15, "DB_CONN_TIMEOUT_SECS")?,
        })
    }

    fn get_env(key: &'static str) -> Result<String> {
        env::var(key).with_context(|| format!("failed to load environment variable {key}"))
    }

    fn get_env_or_default<T>(default: T, key: &'static str) -> Result<T>
    where
        T: FromStr + Display,
        T::Err: Send + Sync + 'static + std::error::Error,
    {
        match env::var(key) {
            Err(VarError::NotUnicode(_)) => Err(anyhow!(
                "environment variable {key} was present but not valid Unicode"
            )),
            Err(VarError::NotPresent) => {
                println!("environment variable {key} not found, using {default}");
                Ok(default)
            }
            Ok(val) => val.parse().with_context(|| {
                format!(
                    "environment variable {key} was present but could not be parsed as {}",
                    type_name::<T>()
                )
            }),
        }
    }
}
