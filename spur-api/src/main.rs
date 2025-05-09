use anyhow::Result;
use axum::{Router, routing::get};
use tokio::net::TcpListener;

const ADDRESS: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(hello_world));
    let listener = TcpListener::bind(ADDRESS).await?;

    println!("Listening on http://{}...", ADDRESS);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello_world() -> &'static str { "Hello, World!" }
