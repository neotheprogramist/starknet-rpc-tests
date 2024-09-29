use reqwest::{get, Client};
use starknet_types_core::felt::Felt;
use url::Url;

use super::{
    errors::DevnetError,
    models::{
        AbortBlocksParams, AbortBlocksResponse, AccountBalanceParams, AccountBalanceResponse,
        CreateBlockResponse, DevnetConfigResponse, DumpPath, IncreaseTimeParams,
        IncreaseTimeResponse, LoadPath, MintTokensParams, MintTokensResponse, MsgToL2,
        PostmanFlushParameters, PostmanFlushResponse, PostmanLoadL1MessagingContractParams,
        PostmanLoadL1MessagingContractResponse, PostmanSendMessageToL2Response,
        SerializableAccount, SetTimeParams, SetTimeResponse,
    },
};

pub async fn is_alive(url: Url) -> Result<String, DevnetError> {
    let response_text = get(url.join("is_alive")?).await?.text().await?;

    Ok(response_text)
}

pub async fn get_account_balance(
    url: Url,
    balance_params: AccountBalanceParams,
) -> Result<AccountBalanceResponse, DevnetError> {
    let response = Client::new()
        .get(url.join("account_balance")?)
        .query(&balance_params)
        .send()
        .await?
        .json::<AccountBalanceResponse>()
        .await?;

    Ok(response)
}

pub async fn get_predeployed_accounts(url: Url) -> Result<Vec<SerializableAccount>, DevnetError> {
    let response = Client::new()
        .get(url.join("predeployed_accounts")?)
        .send()
        .await?
        .json::<Vec<SerializableAccount>>()
        .await?;

    Ok(response)
}

pub async fn dump(url: Url, dump_path: DumpPath) -> Result<(), DevnetError> {
    Client::new()
        .post(url.join("dump")?)
        .json(&dump_path)
        .send()
        .await?;

    Ok(())
}

pub async fn load(url: Url, dump_path: LoadPath) -> Result<(), DevnetError> {
    Client::new()
        .post(url.join("load")?)
        .json(&dump_path)
        .send()
        .await?;

    Ok(())
}

pub async fn postman_load_l1_messaging_contract(
    url: Url,
    load_l1_messaging_contract: PostmanLoadL1MessagingContractParams,
) -> Result<PostmanLoadL1MessagingContractResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("postman/load_l1_messaging_contract")?)
        .json(&load_l1_messaging_contract)
        .send()
        .await?
        .json::<PostmanLoadL1MessagingContractResponse>()
        .await?;

    Ok(response)
}

pub async fn set_time(
    url: Url,
    set_time_params: SetTimeParams,
) -> Result<SetTimeResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("set_time")?)
        .json(&set_time_params)
        .send()
        .await?
        .json::<SetTimeResponse>()
        .await?;

    Ok(response)
}

pub async fn increase_time(
    url: Url,
    increase_time_params: IncreaseTimeParams,
) -> Result<IncreaseTimeResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("increase_time")?)
        .json(&increase_time_params)
        .send()
        .await?
        .json::<IncreaseTimeResponse>()
        .await?;

    Ok(response)
}

pub async fn mint(
    url: Url,
    mint_params: MintTokensParams,
) -> Result<MintTokensResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("mint")?)
        .json(&mint_params)
        .send()
        .await?
        .json::<MintTokensResponse>()
        .await?;

    Ok(response)
}

pub async fn create_block(url: Url) -> Result<CreateBlockResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("create_block")?)
        .send()
        .await?
        .json::<CreateBlockResponse>()
        .await?;

    Ok(response)
}

pub async fn abort_blocks(
    url: Url,
    abort_blocks_params: AbortBlocksParams,
) -> Result<AbortBlocksResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("abort_blocks")?)
        .json(&abort_blocks_params)
        .send()
        .await?
        .json::<AbortBlocksResponse>()
        .await?;

    Ok(response)
}

pub async fn restart(url: Url) -> Result<(), DevnetError> {
    let response = Client::new().post(url.join("restart")?).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(DevnetError::Restart {
            msg: response.text().await?,
        })
    }
}

pub async fn postman_flush(
    url: Url,
    postman_flush_params: PostmanFlushParameters,
) -> Result<PostmanFlushResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("postman/flush")?)
        .json(&postman_flush_params)
        .send()
        .await?
        .json::<PostmanFlushResponse>()
        .await?;

    Ok(response)
}

pub async fn postman_send_message_to_l2(
    url: Url,
    postman_send_message_to_l2_params: MsgToL2<Felt>,
) -> Result<PostmanSendMessageToL2Response, DevnetError> {
    let response = Client::new()
        .post(url.join("postman/send_message_to_l2")?)
        .json(&postman_send_message_to_l2_params)
        .send()
        .await?
        .json::<PostmanSendMessageToL2Response>()
        .await?;

    Ok(response)
}

pub async fn devnet_config(url: Url) -> Result<DevnetConfigResponse, DevnetError> {
    let response = Client::new()
        .get(url.join("config")?)
        .send()
        .await?
        .json::<DevnetConfigResponse>()
        .await?;

    Ok(response)
}
