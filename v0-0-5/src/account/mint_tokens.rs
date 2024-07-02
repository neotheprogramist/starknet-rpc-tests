use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use starknet_core::types::Felt;
use tracing::info;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct MintRequest {
    amount: u128,
    address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Unit {
    WEI,
    SRI,
}

#[derive(Serialize, Deserialize, Debug)]
struct MintResponse {
    new_balance: String,
    unit: Unit,
    tx_hash: Felt,
}

pub async fn mint_tokens(base_url: Url, mint_request: &MintRequest) -> Result<(), reqwest::Error> {
    let mint_url = base_url.join("mint").unwrap();
    info!("{} {:?}", "Minting tokens at".green(), mint_url);
    let response = Client::new()
        .post(mint_url)
        .header("Content-type", "application/json")
        .json(mint_request)
        .send()
        .await?;

    match response.status().is_success() {
        true => {
            let mint_response: MintResponse = response.json().await?;
            info!(
                "{} {:?}",
                "Compatible - Token minting successful".green(),
                mint_response,
            );
            Ok(())
        }
        false => {
            info!(
                "{} {}",
                "Incompatible - Token minting unsuccessful.".red(),
                response.status().to_string().red()
            );
            Ok(())
        }
    }
}
