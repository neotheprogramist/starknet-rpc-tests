mod args;
use args::Args;
use clap::Parser;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    info!("args {:?}", args);
    Ok(())
}
