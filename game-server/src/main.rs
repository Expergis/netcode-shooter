//! `game-server` entry point skeleton.

mod rewind;
mod ticker;

use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();

    let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    info!("Binding 128Hz Shooter Game Server on {addr}");

    let socket = tokio::net::UdpSocket::bind(addr).await?;

    info!("Server engine initialized – starting 128 Hz loop");
    ticker::ServerTicker::new(socket).run().await;

    Ok(())
}
