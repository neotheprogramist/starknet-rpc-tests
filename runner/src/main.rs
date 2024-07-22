mod args;
use args::Args;
use clap::Parser;
use colored::*;
use tracing::{error, info};
use versions::v5::{
    devnet::test_devnet_endpoints,
    rpc::{
        accounts::creation::create::{create, AccountType},
        providers::jsonrpc::{HttpTransport, JsonRpcClient},
    },
};

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    match test_devnet_endpoints(args.url.clone()).await {
        Ok(_) => {}
        Err(e) => error!("Failure: {}", e.to_string().red()),
    };

    let provider = JsonRpcClient::new(HttpTransport::new(args.url.clone()));
    let create_acc_data = match create(&provider, AccountType::Oz, Option::None, Option::None).await
    {
        Ok(value) => value,
        Err(e) => {
            info!("{}", "Could not create an account".red());
            return Err(e.to_string());
        }
    };

    Ok(())
}
