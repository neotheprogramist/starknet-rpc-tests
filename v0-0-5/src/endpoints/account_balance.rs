use reqwest::Client;
use serde::{Deserialize, Serialize};
use starknet_core::types::Felt;
use std::error::Error as StdError;
use std::fmt;
use url::Url;
#[derive(Deserialize, Debug)]
pub struct AccountBalanceResponse {
    pub amount: (u64, u64, u64),
    pub unit: String,
}

#[derive(Serialize)]
pub struct AccountBalanceParams {
    // #[serde(serialize_with = "crate::serialize_felt_to_hex::serialize_field_element")]
    pub address: Felt,
    pub unit: String,
    pub block_tag: String,
}

pub async fn account_balance(
    account_balance_params: &AccountBalanceParams,
    base_url: Url,
) -> Result<AccountBalanceResponse, RequestOrParseError> {
    let client: Client = Client::new();
    let account_balance_url = match base_url.join("account_balance") {
        Ok(url) => url,
        Err(e) => return Err(e.into()),
    };
    let res = client
        .get(account_balance_url)
        .query(account_balance_params)
        .send()
        .await?;

    match res.json::<AccountBalanceResponse>().await {
        Ok(account) => Ok(account),
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug)]
pub enum RequestOrParseError {
    Reqwest(reqwest::Error),
    Url(url::ParseError),
}

impl fmt::Display for RequestOrParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestOrParseError::Reqwest(e) => write!(f, "{}", e),
            RequestOrParseError::Url(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for RequestOrParseError {}

impl From<reqwest::Error> for RequestOrParseError {
    fn from(err: reqwest::Error) -> RequestOrParseError {
        RequestOrParseError::Reqwest(err)
    }
}

impl From<url::ParseError> for RequestOrParseError {
    fn from(err: url::ParseError) -> RequestOrParseError {
        RequestOrParseError::Url(err)
    }
}
