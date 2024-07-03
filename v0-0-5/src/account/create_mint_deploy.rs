use crate::{
    endpoints::mint::{mint, MintRequest},
    jsonrpc::{HttpTransport, JsonRpcClient},
};
use colored::*;
use tracing::info;
use url::Url;

use super::{
    create::{create, get_chain_id, AccountCreateResponse, AccountType},
    deploy::{deploy, Deploy, ValidatedWaitParams, WaitForTx},
};

pub async fn create_mint_deploy(url: Url) -> Result<AccountCreateResponse, String> {
    let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_account_data = match create(&jsonrpc_client, AccountType::Oz, Option::None).await {
        Ok(value) => {
            info!("{}", format!("{:?}", value.account_data).green());
            value
        }
        Err(e) => {
            info!("{}", "Could not create an account".red());
            return Err(e);
        }
    };

    match mint(
        url,
        &MintRequest {
            amount: u128::MAX,
            address: create_account_data.account_data.address,
        },
    )
    .await
    {
        Ok(response) => info!("{} {:?}", "Minted tokens".green(), response),
        Err(e) => {
            info!("{}", "Could not mint tokens".red());
            return Err(e.to_string());
        }
    };

    let deploy_args = Deploy {
        name: None,
        max_fee: Some(create_account_data.max_fee),
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&jsonrpc_client).await?;
    match deploy(
        &jsonrpc_client,
        deploy_args,
        chain_id,
        wait_conifg,
        create_account_data.clone(),
    )
    .await
    {
        Ok(value) => {
            info!("{}", format!("{:?}", value).green());
            Some(value)
        }
        Err(e) => {
            info!("{}", "Could not deploy an account".red());
            return Err(e);
        }
    };
    Ok(create_account_data)
}
