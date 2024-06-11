mod args;
mod errors;
mod transports;
mod utils;
use args::Args;
use clap::Parser;
mod tests;
use shared::account_balance::{account_balance, AccountBalanceParams};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    let account_balance_params = AccountBalanceParams {
        address: "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691".to_string(),
        unit: "WEI".to_string(),
        block_tag: "latest".to_string(),
    };
    account_balance(&account_balance_params, &args.vers, args.url).await?;
    Ok(())
}
