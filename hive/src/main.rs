mod args;
use args::Args;
use clap::Parser;
use colored::Colorize;
use openrpc_checker::hive::test_rpc_endpoints;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    info!("args {:?}", args);
    if let Err(e) = test_rpc_endpoints(
        args.url_list,
        &args.sierra_path,
        &args.casm_path,
        &args.sierra_path_2,
        &args.casm_path_2,
        args.account_class_hash,
        args.account_address,
        args.private_key,
        args.erc20_strk_contract_address,
        args.erc20_eth_contract_address,
        args.amount_per_test,
    )
    .await
    {
        error!("Failure: {}", e.to_string().red());
    }

    Ok(())
}
