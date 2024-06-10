mod args;
mod errors;
mod transports;
mod utils;
use args::Args;
use clap::Parser;
mod tests;
use shared::account_balance::{account_balance, AccountBalanceParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _args: Args = Args::parse();
    let account_balance_params = AccountBalanceParams {
        address: "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691".to_string(),
        unit: "WEI".to_string(),
        block_tag: "latest".to_string(),
    };
    account_balance(&account_balance_params, &_args.vers, &_args.url).await?;

    Ok(())
}
