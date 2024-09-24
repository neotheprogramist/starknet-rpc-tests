use reqwest::Client;
use url::Url;

use super::{
    errors::DevnetError,
    models::{
        AbortBlocksParams, AbortBlocksResponse, AccountBalanceParams, AccountBalanceResponse,
        CreateBlockResponse, ForkStatusResponse, IncreaseTimeParams, IncreaseTimeResponse,
        MintTokensParams, MintTokensResponse, SerializableAccount, SetTimeParams, SetTimeResponse,
    },
};

pub async fn is_alive(url: Url) -> Result<String, DevnetError> {
    let response_text = Client::new()
        .get(url.join("is_alive")?)
        .send()
        .await?
        .text()
        .await?;

    Ok(response_text)
}

pub async fn restart(url: Url) -> Result<(), DevnetError> {
    let response = Client::new().post(url.join("restart")?).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(DevnetError::RestartError {
            msg: response.text().await?,
        })
    }
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

pub async fn set_time(url: Url, params: SetTimeParams) -> Result<SetTimeResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("set_time")?)
        .json(&params)
        .send()
        .await?
        .json::<SetTimeResponse>()
        .await?;

    Ok(response)
}

pub async fn increase_time(
    url: Url,
    params: IncreaseTimeParams,
) -> Result<IncreaseTimeResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("increase_time")?)
        .json(&params)
        .send()
        .await?
        .json::<IncreaseTimeResponse>()
        .await?;

    Ok(response)
}

pub async fn mint(url: Url, params: MintTokensParams) -> Result<MintTokensResponse, DevnetError> {
    let response = Client::new()
        .post(url.join("mint")?)
        .json(&params)
        .send()
        .await?
        .json::<MintTokensResponse>()
        .await?;

    Ok(response)
}

pub async fn fork_status(url: Url) -> Result<ForkStatusResponse, DevnetError> {
    let response = Client::new()
        .get(url.join("fork_status")?)
        .send()
        .await?
        .json::<ForkStatusResponse>()
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
