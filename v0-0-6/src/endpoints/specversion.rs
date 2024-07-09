use colored::*;
use tracing::info;
use url::Url;

use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::Provider,
};

use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecVersionResponse {
    version: String,
}

pub async fn specversion(url: Url) -> Result<SpecVersionResponse, Box<dyn Error>> {
    let client = JsonRpcClient::new(HttpTransport::new(url));

    let response = client.spec_version().await?;

    let parsed_response = SpecVersionResponse { version: response };
    info!("{}", "SpecVersion Compatible".green());
    Ok(parsed_response)
}
