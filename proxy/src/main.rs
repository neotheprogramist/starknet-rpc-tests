mod core;
use clap::Parser;
use tracing::info;

use core::errors::ProxyError;
use core::utils::{handle_connection, load_tls_config};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), ProxyError> {
    colored::control::set_override(true);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));

    let listener = TcpListener::bind(addr).await?;

    let tls_config = load_tls_config()?;

    info!("Proxy server is running on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let tls_config = tls_config.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, tls_config).await {
                eprintln!("Error handling connection: {:?}", e);
            }
        });
    }
}
