use serde::Deserialize;

use tracing::info;

use std::error::Error as StdError;
use std::fmt;
use url::Url;

use crate::account::create_mint_deploy::create_mint_deploy;
use crate::jsonrpc::{HttpTransport, JsonRpcClient};
use crate::provider::{Provider, ProviderError};

use colored::*;

use super::mint::FeeUnit;
#[derive(Deserialize, Debug)]
pub struct AccountBalanceResponse {
    pub amount: Vec<u64>,
    pub unit: String,
}

pub async fn account_balance(base_url: Url) -> Result<AccountBalanceResponse, RequestOrParseError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(base_url.clone()));
    let acc = create_mint_deploy(base_url.clone()).await.unwrap();
    let response = rpc_client
        .account_balance(
            acc.account_data.address,
            FeeUnit::WEI,
            starknet_core::types::BlockTag::Latest,
        )
        .await;
    match response {
        Ok(value) => match serde_json::from_value::<AccountBalanceResponse>(value) {
            Ok(account_balance_respnose) => {
                info!("{}", "Account Balance Compatible".green());
                Ok(account_balance_respnose)
            }
            Err(e) => {
                info!("{}", "Account Balance Incompatible".red());
                Err(RequestOrParseError::Serde(e))
            }
        },
        Err(e) => Err(RequestOrParseError::from(e)),
    }
}

#[derive(Debug)]
pub enum RequestOrParseError {
    Reqwest(reqwest::Error),
    Url(url::ParseError),
    Serde(serde_json::Error),
    Provider(ProviderError),
}

impl fmt::Display for RequestOrParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestOrParseError::Reqwest(e) => write!(f, "{}", e),
            RequestOrParseError::Url(e) => write!(f, "{}", e),
            RequestOrParseError::Serde(e) => write!(f, "{}", e),
            RequestOrParseError::Provider(e) => write!(f, "Provider error: {:?}", e),
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

impl From<ProviderError> for RequestOrParseError {
    fn from(err: ProviderError) -> RequestOrParseError {
        RequestOrParseError::Provider(err)
    }
}
