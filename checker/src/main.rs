mod args;
mod v5;
mod v6;
mod v7;
use args::{Args, Version};
use clap::Parser;
use colored::*;
use tracing::error;
use v5 as V5;
use v6 as V6;
use v7 as V7;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    match args.version {
        Version::V5 => {
            if let Err(e) = V5::devnet::test_devnet_endpoints(
                args.url.clone(),
                args.l1_network_url.clone(),
                &args.sierra_path,
                &args.casm_path,
            )
            .await
            {
                error!("Failure: {}", e.to_string().red());
            }

            if let Err(e) = V5::rpc::endpoints::test_rpc_endpoints(
                args.url.clone(),
                &args.sierra_path,
                &args.casm_path,
            )
            .await
            {
                error!("Failure: {}", e.to_string().red());
            }
        }
        Version::V6 => {
            if let Err(e) = V6::devnet::test_devnet_endpoints(
                args.url.clone(),
                args.l1_network_url.clone(),
                &args.sierra_path,
                &args.casm_path,
            )
            .await
            {
                error!("Failure: {}", e.to_string().red());
            }

            if let Err(e) = V6::rpc::endpoints::test_rpc_endpoints_v0_0_6(
                args.url.clone(),
                &args.sierra_path,
                &args.casm_path,
            )
            .await
            {
                error!("Failure: {}", e.to_string().red());
            }
        }
        Version::V7 => {
            if let Err(e) = V7::devnet::test_devnet_endpoints(args.url.clone()).await {
                error!("Failure: {}", e.to_string().red());
            }

            if let Err(e) = V7::rpc::endpoints::test_rpc_endpoints_v0_0_7(
                args.url.clone(),
                &args.sierra_path,
                &args.casm_path,
            )
            .await
            {
                error!("Failure: {}", e.to_string().red());
            }
        }
    }

    Ok(())
}
