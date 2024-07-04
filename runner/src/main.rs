mod args;

use v0_0_5::account::create_mint_deploy::create_mint_deploy;
use v0_0_5::endpoints::specversion::run;

use args::Args;
use clap::Parser;
use colored::Colorize;
use rand::Rng;
use shared::{
    clients::devnet_client::DevnetClient,
    create_acc::{create, get_chain_id, AccountCreateResponse, AccountType},
    deploy_acc::{deploy, Deploy, ValidatedWaitParams, WaitForTx},
};
use starknet_crypto::FieldElement;
use starknet_signers::{LocalWallet, SigningKey};
use tracing::info;
use url::Url;
use utils::{
    account::{
        single_owner::{ExecutionEncoding, SingleOwnerAccount},
        Account, ConnectedAccount,
    },
    codegen::BlockTag,
    models::FeeUnit,
    provider::Provider,
    transports::http::HttpTransport,
};

use starknet_providers::jsonrpc::HttpTransport as StarknetHttpTransport;
use starknet_providers::JsonRpcClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();
    run(args.url.clone()).await;

    create_mint_deploy(Url::parse(args.url.as_ref())?).await?;

    // let client = DevnetClient::new(HttpTransport::new(Url::parse(args.url.as_ref())?));

    // First of all create and deploy an account:
    // let account_create_response = create_and_deploy_account(&args).await?;
    // info!("{:?}", account_create_response.account_data);

    // let account = SingleOwnerAccount::new(
    //     client,
    //     LocalWallet::from(SigningKey::from_secret_scalar(
    //         account_create_response.account_data.private_key,
    //     )),
    //     account_create_response.account_data.address,
    //     FieldElement::from_hex_be(&args.chain_id).unwrap(),
    //     ExecutionEncoding::New,
    // );

    // let account = SingleOwnerAccount::new(
    //     client,
    //     LocalWallet::from(SigningKey::from_secret_scalar(args.private_key)),
    //     args.account_address,
    //     FieldElement::from_hex_be(&args.chain_id).unwrap(),
    //     ExecutionEncoding::New,
    // );

    // match account.provider().get_predeployed_accounts().await {
    //     Ok(value) => {
    //         info!("{}", "Get predeployed accounts COMPATIBLE".green());
    //         println!("{:?}", value);
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }
    // match account.provider().get_config().await {
    //     Ok(value) => {
    //         info!("{}", "Get config COMPATIBLE".green());
    //         println!("{:?}", value);
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }

    // match account
    //     .provider()
    //     .get_account_balance(
    //         account_create_response.account_data.address,
    //         FeeUnit::WEI,
    //         BlockTag::Latest,
    //     )
    //     .await
    // {
    //     Ok(value) => {
    //         info!("{}", "get account balance COMPATIBLE".green());
    //         println!("{:?}", value);
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }
    // match account
    //     .provider()
    //     .mint(account_create_response.account_data.address, 1000)
    //     .await
    // {
    //     Ok(value) => {
    //         info!("{}", "COMPATIBLE".green());
    //         println!("{:?}", value);
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }
    // match account.provider().set_time(100, false).await {
    //     Ok(value) => {
    //         info!("{}", "COMPATIBLE".green());
    //         println!("{:?}", value);
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }

    // match account.provider().increase_time(1000).await {
    //     Ok(value) => {
    //         info!("{}", "COMPATIBLE".green());
    //         println!("{:?}", value);
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }

    // match account.provider().create_block().await {
    //     Ok(value) => {
    //         info!("{}", "COMPATIBLE".green());
    //         if let Some(block_hash) = value.get("block_hash").and_then(|v| v.as_str()) {
    //             println!("Block hash: {}", block_hash);
    //             match account
    //                 .provider()
    //                 .abort_blocks(block_hash.to_string())
    //                 .await
    //             {
    //                 Ok(value) => {
    //                     info!("{}", "COMPATIBLE".green());
    //                     println!("{:?}", value);
    //                 }
    //                 Err(_) => info!("{}", "INCOMPATIBLE".red()),
    //             }
    //         } else {
    //             println!("Block hash not found");
    //         }
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }

    Ok(())
}

async fn fuzzy_test_mint(
    account: &SingleOwnerAccount<DevnetClient<HttpTransport>, LocalWallet>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let test_count = rng.gen_range(5..=20);

    for _ in 0..test_count {
        let initial_balance = account
            .provider()
            .get_account_balance(account.address(), FeeUnit::WEI, BlockTag::Latest)
            .await?;

        let mint_amount = rng.gen_range(u128::MIN + 1..=u128::MAX);

        let mint_result = account
            .provider()
            .mint(account.address(), mint_amount)
            .await?;

        let new_balance = account
            .provider()
            .get_account_balance(account.address(), FeeUnit::WEI, BlockTag::Latest)
            .await?;
    }

    Ok(())
}
