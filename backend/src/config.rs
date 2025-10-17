use anyhow::{Context, Result, anyhow};
use colored::Colorize;
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

        // RUST_LOG is not used directly in this config, but is necessary for logging
        Self::check_rust_log();

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
            Err(VarError::NotPresent) => {
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

    /// Checks whether the environment variable `RUST_LOG` is present, valid Unicode, and a valid
    /// value. If valid, logs the log level using the "info" log level. Otherwise, prints the
    /// problem to stderr (since logging cannot be used to log the error if `RUST_LOG` is not
    /// valid).
    fn check_rust_log() {
        match env::var("RUST_LOG") {
            Err(VarError::NotPresent) => eprintln!(
                "{}",
                "Environment variable RUST_LOG not found. Logging will not work.".red()
            ),
            Err(VarError::NotUnicode(_)) => eprintln!(
                "{}",
                "Environment variable RUST_LOG present but not valid Unicode. \
                Logging will not work."
                    .red()
            ),
            Ok(val) => {
                let level = val.to_ascii_lowercase();

                match level.as_str() {
                    "error" | "warn" | "info" | "debug" | "trace" | "off" => {
                        log::info!("Log level set to {level}");
                    }
                    _ => eprintln!(
                        "{}",
                        "Environment variable RUST_LOG present but not a valid value. \
                        Logging will not work. \
                        Valid values are error, warn, info, debug, trace, and off."
                            .red()
                    ),
                }
            }
        }
    }
}
