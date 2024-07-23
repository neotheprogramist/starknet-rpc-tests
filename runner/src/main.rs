mod args;
use args::Args;
use clap::Parser;
use colored::*;
use tracing::{error, info};
use versions::v5::{
    devnet::test_devnet_endpoints,
    rpc::{
        accounts::{
            creation::{
                create::{create, AccountType},
                helpers::get_chain_id,
                structs::MintRequest,
            },
            deployment::{
                deploy::deploy,
                structs::{ValidatedWaitParams, WaitForTx},
            },
            utils::mint::mint,
        },
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
    info!("{:?}", create_acc_data);
    match mint(
        args.url.clone(),
        &MintRequest {
            amount: u128::MAX,
            address: create_acc_data.address,
        },
    )
    .await
    {
        Ok(response) => info!("{} {} {:?}", "Minted tokens".green(), u128::MAX, response),
        Err(e) => {
            info!("{}", "Could not mint tokens".red());
            return Err(e.to_string());
        }
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&provider).await.unwrap();
    info! {"Before deploy"}
    let result = match deploy(provider, chain_id, wait_conifg, create_acc_data.clone()).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account".red());
            return Err(e.to_string());
        }
    };
    info!("After deploy, resultt: {:?}", result);

    Ok(())
}
