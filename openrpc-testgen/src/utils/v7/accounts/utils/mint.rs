use reqwest::{Client, StatusCode};

use thiserror::Error;
use url::Url;

use crate::utils::v7::accounts::creation::structs::{MintRequest2, MintResponse};

#[derive(Error, Debug)]
pub enum MintError {
    #[error(transparent)]
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

    #[error(transparent)]
    JoinUrlError(#[from] url::ParseError),
}

pub async fn mint(base_url: Url, mint_request: &MintRequest2) -> Result<MintResponse, MintError> {
    let mint_url = match base_url.join("mint") {
        Ok(url) => url,
        Err(e) => return Err(MintError::JoinUrlError(e)),
    };

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
