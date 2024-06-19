mod args;

use args::Args;
use clap::Parser;
use colored::Colorize;
use shared::clients::devnet_client::DevnetClient;
use starknet_crypto::FieldElement;
use starknet_signers::{LocalWallet, SigningKey};
use tracing::info;
use url::Url;
use utils::{
    account::{
        single_owner::{ExecutionEncoding, SingleOwnerAccount},
        ConnectedAccount,
    },
    codegen::BlockTag,
    models::FeeUnit,
    provider::Provider,
    transports::http::HttpTransport,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();
    let client = DevnetClient::new(HttpTransport::new(Url::parse(&args.url.to_string())?));
    let account = SingleOwnerAccount::new(
        client,
        LocalWallet::from(SigningKey::from_secret_scalar(args.private_key)),
        args.account_address,
        FieldElement::from_hex_be(&args.chain_id).unwrap(),
        ExecutionEncoding::New,
    );

    match account.provider().get_predeployed_accounts().await {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }
    match account.provider().get_config().await {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }

    match account
        .provider()
        .get_account_balance(args.account_address, FeeUnit::WEI, BlockTag::Latest)
        .await
    {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }
    match account.provider().mint(args.account_address, 1000).await {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }
    match account.provider().set_time(100, false).await {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }

    match account.provider().increase_time(1000).await {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }

    Ok(())
}
