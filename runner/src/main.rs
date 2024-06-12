mod args;

use args::Args;
use clap::Parser;
use shared::account_balance::{account_balance, AccountBalanceParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    let account_balance_params = AccountBalanceParams {
        address: args.account_address,
        unit: "WEI".to_string(),
        block_tag: "latest".to_string(),
    };
    account_balance(&account_balance_params, &args.vers, args.url).await?;
    Ok(())
}
