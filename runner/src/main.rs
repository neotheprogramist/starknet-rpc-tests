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

    match account
        .provider()
        .load("http://localhost:8545".to_string(), Option::None)
        .await
    {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }

    match account
        .provider()
        .send_message_to_l2(
            FieldElement::from_hex_be(
                "0x00285ddb7e5c777b310d806b9b2a0f7c7ba0a41f12b420219209d97a3b7f25b2",
            )
            .unwrap(),
            FieldElement::from_hex_be(
                "0xC73F681176FC7B3F9693986FD7B14581E8D540519E27400E88B8713932BE01",
            )
            .unwrap(),
            FieldElement::from_hex_be("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512").unwrap(),
            vec![
                FieldElement::from_hex_be("0x1").unwrap(),
                FieldElement::from_hex_be("0x2").unwrap(),
            ],
            FieldElement::from_hex_be("0x123456abcdef").unwrap(),
            FieldElement::from_hex_be("0x0").unwrap(),
        )
        .await
    {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }
    Ok(())
}
