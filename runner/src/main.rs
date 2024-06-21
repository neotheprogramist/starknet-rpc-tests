mod args;
pub mod declare;
pub mod deploy;
pub mod tests;
use args::Args;
use clap::Parser;
use colored::Colorize;
use shared::clients::devnet_client::DevnetClient;
use starknet_crypto::FieldElement;
use starknet_signers::{LocalWallet, SigningKey};
use tests::tests::declare_and_deploy;
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
    match account
        .provider()
        .mint(format!("0x{:x}", args.account_address), 1000)
        .await
    {
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
        .load("http://127.0.0.1:8545".to_string(), Option::None)
        .await
    {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }
    let (account, contract_address) = declare_and_deploy(
        "0x4b3f4ba8c00a02b66142a4b1dd41a4dfab4f92650922a3280977b0f03c75ee1",
        "0x57b2f8431c772e647712ae93cc616638",
        "0x534e5f5345504f4c4941",
        "../../target/dev/example_HelloStarknet.contract_class.json",
        "../../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await;

    match account
        .provider()
        .send_message_to_l2(
            contract_address.to_string(),
            "get_balance".to_string(),
            "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512".to_string(),
            vec![],
            "0x123456abcdef".to_string(),
            "0x0".to_string(),
        )
        .await
    {
        Ok(value) => {
            info!("{}", "COMPATIBLE".green());
            println!("{:?}", value);
        }
        Err(_) => info!("{}", "INCOMPATIBLE".red()),
    }

    // match account
    //     .provider()
    //     .consume_message_from_l2(

    //     )
    //     .await
    // {
    //     Ok(value) => {
    //         info!("{}", "COMPATIBLE".green());
    //         println!("{:?}", value);
    //     }
    //     Err(_) => info!("{}", "INCOMPATIBLE".red()),
    // }
    Ok(())
}
