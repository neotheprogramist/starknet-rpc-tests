use reqwest::{get, Client};
use url::Url;

use super::{
    errors::DevnetError,
    models::{
        AccountBalanceParams, AccountBalanceResponse, DumpPath, LoadPath, SerializableAccount,
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

pub async fn dump(url: Url, params: DumpPath) -> Result<(), DevnetError> {
    Client::new()
        .post(url.join("dump")?)
        .json(&params)
        .send()
        .await?;

    Ok(())
}

pub async fn load(url: Url, params: LoadPath) -> Result<(), DevnetError> {
    Client::new()
        .post(url.join("load")?)
        .json(&params)
        .send()
        .await?;

    Ok(())
}
