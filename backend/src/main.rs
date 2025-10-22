#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod api;
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

use crate::api::router;
use anyhow::Result;
use config::AppConfig;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    spur::logger::init_with_default(log::LevelFilter::Info);
    log::info!("Initializing app...");

    let config = AppConfig::load()?;
    let state = AppState::init(&config).await?;
    let app = router::build(state, &config.frontend_url)?;
    let listener = TcpListener::bind(&config.bind_addr).await?;

    #[cfg(debug_assertions)]
    log::info!(
        "Development server listening on http://{}",
        &config.bind_addr
    );

    #[cfg(not(debug_assertions))]
    log::info("Listening on {}", &config.bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal_handler()?)
        .await?;

    Ok(())
}

/// Creates Unix signal handlers that listen for SIGINT and SIGTERM. Errors installing the handlers
/// are returned synchronously, while errors receiving the signals are logged but not returned
/// (since the app would be shutting down anyway). This program will likely only ever run on Unix,
/// but see the other function of the same name for non-Unix behavior.
#[cfg(unix)]
fn shutdown_signal_handler() -> Result<impl Future<Output = ()>> {
    use tokio::signal::unix;

    // Initialize here synchronously to fail fast and separate initialization errors from stream
    // errors
    let mut sigint = unix::signal(unix::SignalKind::interrupt())?;
    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;

    Ok(async move {
        tokio::select! {
            v = sigint.recv() => {
                match v {
                    Some(()) => log::info!("SIGINT received, shutting down..."),
                    None => log::warn!("SIGINT stream ended unexpectedly, shutting down..."),
                }
            }
            v = sigterm.recv() => {
                match v {
                    Some(()) => log::info!("SIGTERM received, shutting down..."),
                    None => log::warn!("SIGTERM stream ended unexpectedly, shutting down..."),
                }
            }
        }
    })
}

/// Creates a cross-platform signal handler that listens for Ctrl+C. Included for portability, but
/// will likely never be used. Not actually tested on Windows. See the other function of the same
/// name for the standard Unix behavior.
#[allow(clippy::unnecessary_wraps)] // Wrapped in `Result` to match the Unix version
#[cfg(not(unix))]
fn shutdown_signal_handler() -> Result<impl Future<Output = ()>> {
    Ok(async {
        match tokio::signal::ctrl_c().await {
            Err(e) => log::error!("Ctrl+C handler error, shutting down: {e}"),
            Ok(()) => log::info!("Ctrl+C received, shutting down..."),
        }
    })
}
