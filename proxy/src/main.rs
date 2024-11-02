mod core;
pub mod test_cases;
use clap::Parser;

use core::errors::ProxyError;
use core::utils::{handle_connection, load_tls_config};
use std::net::TcpListener;

use std::net::SocketAddr;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

fn main() -> Result<(), ProxyError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let cli = Cli::parse();

    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));

    let listener = TcpListener::bind(addr)?;

    let tls_config = load_tls_config()?;

    for stream in listener.incoming() {
        let stream = stream?;

        handle_connection(stream, tls_config.clone())?;
    }
    Ok(())
}
