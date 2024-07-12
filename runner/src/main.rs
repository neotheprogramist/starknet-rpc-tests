mod args;
use args::Args;
use clap::Parser;
use colored::*;
use tracing::error;
use versions::v5::devnet::test_devnet_endpoints;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    match test_devnet_endpoints(args.url.clone()).await {
        Ok(_) => {}
        Err(e) => error!("Failure: {}", e.to_string().red()),
    };
}
