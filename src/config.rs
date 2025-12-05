use anyhow::{Context, Result, anyhow};
use std::{
    any::type_name,
    env::{self, VarError},
    fmt::Display,
    str::FromStr,
};

pub struct AppConfig {
    pub database_url: String,
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
            jwt_secret: Self::get_env("JWT_SECRET")?,

            // Possibly the same in dev and prod?
            // -> Hardcoded defaults that can be overridden with environment variables
            bind_addr: Self::get_env_or_else(|| String::from("0.0.0.0:8080"), "BIND_ADDR")?,
            max_pool_connections: Self::get_env_or_else(|| 10, "MAX_POOL_CONNECTIONS")?,
            db_conn_timeout_secs: Self::get_env_or_else(|| 15, "DB_CONN_TIMEOUT_SECS")?,
        })
    }

    fn get_env(key: &'static str) -> Result<String> {
        env::var(key).with_context(|| format!("failed to load environment variable {key}"))
    }

    /// Reads in an environment variable using `key`, or if not found, computes a default from a
    /// closure.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the environment variable is present but is not valid Unicode or cannot be
    /// parsed as `T`.
    fn get_env_or_else<T, F>(op: F, key: &'static str) -> Result<T>
    where
        T: FromStr + Display,
        T::Err: Send + Sync + 'static + std::error::Error,
        F: FnOnce() -> T,
    {
        match env::var(key) {
            Err(VarError::NotPresent) => {
                let default = op();
                log::warn!("Environment variable {key} not found, using {default}");
                Ok(default)
            }
            Err(VarError::NotUnicode(_)) => Err(anyhow!(
                "environment variable {key} present but not valid Unicode"
            )),
            Ok(val) => val.parse().with_context(|| {
                format!(
                    "environment variable {key} present but could not be parsed as {}",
                    type_name::<T>()
                )
            }),
        }
    }
}
