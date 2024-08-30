mod args;
use args::Args;
use clap::Parser;
use colored::*;
use tracing::error;
use versions::v5::{devnet::test_devnet_endpoints, rpc::endpoints::test_rpc_endpoints};
use versions::v7::rpc::endpoints::test_rpc_endpoints_v0_0_7;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    // match test_devnet_endpoints(args.url.clone()).await {
    //     Ok(_) => {}
    //     Err(e) => error!("Failure: {}", e.to_string().red()),
    // };
    match test_rpc_endpoints_v0_0_7(args.url.clone(), &args.sierra_path, &args.casm_path).await {
        Ok(_) => {}
        Err(e) => error!("Failure: {}", e.to_string().red()),
    }

    Ok(())
}