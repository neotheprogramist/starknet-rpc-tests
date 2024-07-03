use colored::*;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use starknet_core::types::Felt;
use thiserror::Error;
use tracing::info;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct MintRequest {
    pub amount: u128,
    pub address: Felt,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Unit {
    WEI,
    SRI,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintResponse {
    new_balance: String,
    unit: Unit,
    tx_hash: String,
}

#[derive(Error, Debug)]
pub enum MintError {
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),

    #[error("Response Status Error")]
    ResponseStatusError {
        status_code: StatusCode,
        message: Option<String>,
    },

    #[error("Error getting response text")]
    ResponseTextError,

    #[error("Error parsing response")]
    ResponseParseError,
}

pub async fn mint(base_url: Url, mint_request: &MintRequest) -> Result<MintResponse, MintError> {
    let mint_url = base_url.join("mint").unwrap();
    info!("{} {:?}", "Minting tokens at".green(), mint_url);
    info!("{} {:?}", "Minting request".green(), mint_request);
    let response = Client::new()
        .post(mint_url)
        .header("Content-type", "application/json")
        .json(mint_request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status_code = response.status();
        let error_message = response
            .text()
            .await
            .map_err(|_| MintError::ResponseTextError)?;
        Err(MintError::ResponseStatusError {
            status_code,
            message: Some(error_message),
        })
    } else {
        let mint_response = response
            .json::<MintResponse>()
            .await
            .map_err(|_| MintError::ResponseParseError)?;
        Ok(mint_response)
    }
}
