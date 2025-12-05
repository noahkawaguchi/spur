use std::{
    env,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

/// Makes a very lightweight attempt to connect to the main server over localhost as a health check.
/// Just like the server, gets the port from the `BIND_ADDR` environment variable or defaults to
/// 8080. Waits for 300 ms unless a different value is provided via the environment variable
/// `HEALTHCHECK_MILLIS`.
fn main() -> Result<(), Box<dyn Error>> {
    // Split off the port from the address or default to 8080
    let port = env::var("BIND_ADDR").map_or(Ok(8080), |val| {
        val.split_once(':')
            .ok_or("failed to split BIND_ADDR on ':'")?
            .1
            .parse()
            .map_err(|e| format!("failed to parse port from BIND_ADDR as u16: {e}"))
    })?;

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    let timeout =
        Duration::from_millis(env::var("HEALTHCHECK_MILLIS").map_or(Ok(300), |val| val.parse())?);

    TcpStream::connect_timeout(&socket, timeout)?;

    Ok(())
}
