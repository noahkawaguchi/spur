#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::{
    env,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

/// Makes a very lightweight attempt to connect to the main server over localhost as a health check.
/// Gets the port from the same `BIND_ADDR` environment variable that the server uses. Waits for 300
/// ms unless a different value is provided via the environment variable `HEALTHCHECK_MILLIS`.
fn main() -> Result<(), Box<dyn Error>> {
    let port = env::var("BIND_ADDR")?
        .split_once(':')
        .ok_or("failed to split BIND_ADDR on ':'")?
        .1
        .parse()?;

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

    let timeout = Duration::from_millis(
        env::var("HEALTHCHECK_MILLIS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(300),
    );

    TcpStream::connect_timeout(&socket, timeout)?;

    Ok(())
}
