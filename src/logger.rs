use env_logger::{DEFAULT_FILTER_ENV, Env};
use log::{Level, LevelFilter, log_enabled};

/// Initializes the logger with the provided default if not set from the environment, checks whether
/// logging has likely been accidentally disabled, and reports the max log level using the INFO log
/// level.
///
/// Specifically, checks for the case where logging is disabled but `env_logger::DEFAULT_FILTER_ENV`
/// (which should be `RUST_LOG`) is set in the environment to something other than OFF (case
/// insensitive), printing a warning message if so. In this case, it is likely that the environment
/// variable's value is malformed, since if it is present but invalid, logging seems to
/// unfortunately be silently disabled.
pub fn init_with_default(default_level: LevelFilter) {
    env_logger::Builder::from_env(Env::default().default_filter_or(default_level.as_str())).init();

    if !log_enabled!(Level::Error)
        && let Ok(val) = std::env::var(DEFAULT_FILTER_ENV)
        && !val.eq_ignore_ascii_case(LevelFilter::Off.as_str())
    {
        eprintln!(
            "Warning: Logging is off but environment variable {DEFAULT_FILTER_ENV} is: {val}"
        );
    }

    log::info!("Max log level set to {}", log::max_level());
}
