use colored::Colorize;
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

async fn specversion(url: Url) -> Result<SpecVersionResponse, Box<dyn Error>> {
    let client = JsonRpcClient::new(HttpTransport::new(url));

    let response = client.spec_version().await?;

    let parsed_response = SpecVersionResponse { version: response };

    Ok(parsed_response)
}

pub async fn run(url: Url) {
    match specversion(url).await {
        Ok(_) => {
            info!("{} - {}", "fuzzy_specversion".green(), "compatible".green());
        }
        Err(_) => {
            info!("{} - {}", "fuzzy_specversion".red(), "incompatible".red());
        }
    }
}
