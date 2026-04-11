pub mod api;
pub mod logger;

pub use config::AppConfig;
pub use state::AppState;

mod app_services;
mod config;
mod domain;
mod infra;
mod map_into;
mod models;
mod read_models;
mod state;

#[cfg(test)]
mod test_utils;

/// Replaces `#[tokio::main]`, propagating errors and not inserting `#[allow(clippy::expect_used)]`.
///
/// Based on the "equivalent code" listed in the docs at
/// <https://docs.rs/tokio/latest/tokio/attr.main.html#using-the-multi-threaded-runtime>
///
/// # Errors
///
/// Returns `Err` if the `.build()` step of creating the Tokio runtime returns `Err`.
pub fn tokio_main<F, E>(f: F) -> Result<(), E>
where
    E: From<std::io::Error>,
    F: Future<Output = Result<(), E>>,
{
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(f)
}
