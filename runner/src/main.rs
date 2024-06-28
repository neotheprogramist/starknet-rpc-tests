mod args;

use args::Args;
use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use shared::{
    clients::devnet_client::DevnetClient,
    create_acc::{create, get_chain_id, mint_tokens, AccountType},
    deploy_acc::{deploy, Deploy, ValidatedWaitParams, WaitForTx},
};
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

use starknet_providers::jsonrpc::HttpTransport as StarknetHttpTransport;
use starknet_providers::JsonRpcClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();
    let client = DevnetClient::new(HttpTransport::new(Url::parse(args.url.as_ref())?));
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

    match account.provider().create_block().await {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            if let Some(block_hash) = value.get("block_hash").and_then(|v| v.as_str()) {
                println!("Block hash: {}", block_hash);
                match account
                    .provider()
                    .abort_blocks(block_hash.to_string())
                    .await
                {
                    Ok(value) => {
                        info!("{}", "COMPATIBLE".green());
                        println!("{:?}", value);
                    }
                    Err(_) => info!("{}", "INCOMPATIBLE".red()),
                }
            } else {
                println!("Block hash not found");
            }
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }
    let jsonrpc_client = JsonRpcClient::new(StarknetHttpTransport::new(args.url.clone()));
    let create_account_data = match create(&jsonrpc_client, AccountType::Oz, Option::None).await {
        Ok(value) => {
            info!("{}", format!("{:?}", value).green());
            Some(value)
        }
        Err(_) => {
            info!("{}", "Could not create an account".red());
            return Ok(());
        }
    };

    let deploy_args = Deploy {
        name: None,
        max_fee: Some(create_account_data.as_ref().unwrap().max_fee),
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&jsonrpc_client).await?;
    let deploy_account_result = match deploy(
        &jsonrpc_client,
        deploy_args,
        chain_id,
        wait_conifg,
        create_account_data.unwrap(),
    )
    .await
    {
        Ok(value) => {
            info!("{}", format!("{:?}", value).green());
            Some(value)
        }
        Err(_) => {
            info!("{}", "Could not deploy an account".red());
            return Ok(());
        }
    };

    Ok(())
}
