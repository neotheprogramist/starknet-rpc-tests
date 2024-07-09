use serde::{Deserialize, Serialize};
use starknet_core::types::Felt;
use tracing::info;

use std::error::Error as StdError;
use std::fmt;
use url::Url;

use crate::jsonrpc::{HttpTransport, JsonRpcClient};
use crate::provider::{Provider, ProviderError};

use colored::*;

use super::mint::FeeUnit;
#[derive(Deserialize, Debug)]
pub struct AccountBalanceResponse {
    pub amount: (u64, u64, u64),
    pub unit: String,
}

#[derive(Serialize)]
pub struct AccountBalanceParams {
    pub address: Felt,
    pub unit: String,
    pub block_tag: String,
}

pub async fn account_balance(base_url: Url) -> Result<AccountBalanceResponse, RequestOrParseError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(base_url.clone()));

    let response = rpc_client
        .account_balance(
            Felt::from_hex("0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691")
                .unwrap(),
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
                info!("{}", "Incompatible".red());
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
