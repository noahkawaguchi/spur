use anyhow::{Result, anyhow};
use colored::Colorize;
use env_logger::Env;
use log::LevelFilter;
use std::env::{self, VarError};

/// Initializes the logger and checks whether the environment variable `RUST_LOG` is present, valid
/// Unicode, and a valid value.
///
/// - If present but invalid, returns `Err`.
/// - If present and valid, logs the log level using the INFO level.
/// - If not present, logs that the default level is being used using the WARN level.
pub fn init_with_default(default_level: LevelFilter) -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or(default_level.to_string()))
        .init();

    match env::var("RUST_LOG") {
        Err(VarError::NotPresent) => {
            log::warn!("Environment variable RUST_LOG not found, using log level {default_level}");
            Ok(())
        }
        Err(VarError::NotUnicode(_)) => Err(anyhow!(
            "{}",
            "Environment variable RUST_LOG present but not valid Unicode. \n\
            Valid values are `error`, `warn`, `info`, `debug`, `trace`, and `off`."
                .red()
        )),
        Ok(val) => {
            let level = val.to_ascii_uppercase();

            match level.as_str() {
                "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE" | "OFF" => {
                    log::info!("Log level set to {level}");
                    Ok(())
                }
                _ => Err(anyhow!(
                    "{}",
                    "Environment variable RUST_LOG present but not a valid value. \n\
                    Valid values are `error`, `warn`, `info`, `debug`, `trace`, and `off`."
                        .red()
                )),
            }
        }
    }
}
