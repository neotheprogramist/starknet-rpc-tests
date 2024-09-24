use reqwest::{get, Client};
use url::Url;

use super::{
    errors::DevnetError,
    models::{
        AccountBalanceParams, AccountBalanceResponse, SerializableAccount, SetTimeParams,
        SetTimeResponse,
    },
};

pub async fn is_alive(url: Url) -> Result<String, DevnetError> {
    let response_text = get(url.join("is_alive")?).await?.text().await?;

    Ok(response_text)
}

pub async fn restart(url: Url) -> Result<(), DevnetError> {
    let client = Client::new();

    let response = client.post(url.join("restart")?).send().await?;

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
    let client = Client::new();

    let response = client
        .post(url.join("set_time")?)
        .json(&params)
        .send()
        .await?
        .json::<SetTimeResponse>()
        .await?;

    Ok(response)
}
