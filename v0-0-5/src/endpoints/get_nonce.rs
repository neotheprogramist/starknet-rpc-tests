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

async fn get_nonce(url: Url) {
    let client = JsonRpcClient::new(HttpTransport::new(url));
    let create_account_data = match create(&jsonrpc_client, AccountType::Oz, Option::None).await {
        Ok(value) => {
            info!("{}", format!("{:?}", value.account_data).green());
            value
        }
        Err(e) => {
            info!("{}", "Could not create an account".red());
            return Err(e);
        }
    };
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
