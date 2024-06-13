use crate::errors::RequestOrParseError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use starknet_crypto::FieldElement;
use url::Url;

#[derive(Serialize, Debug)]
pub struct MintParams {
    #[serde(serialize_with = "crate::serialize_felt_to_hex::serialize_field_element")]
    pub address: FieldElement,
    pub amount: u128,
}

#[derive(Deserialize, Debug)]
pub struct MintResponse {
    pub new_balance: FieldElement,
    pub unit: String,
    pub tx_hash: FieldElement,
}

pub async fn mint(
    mint_params: &MintParams,
    base_url: &Url,
) -> Result<MintResponse, RequestOrParseError> {
    let client = Client::new();

    let mint_url = match base_url.join("mint") {
        Ok(url) => url,
        Err(e) => return Err(e.into()),
    };

    let res = match client.post(mint_url).json(mint_params).send().await {
        Ok(res) => res,
        Err(e) => return Err(e.into()),
    };
    let mint_response = match res.json::<MintResponse>().await {
        Ok(mint_response) => mint_response,
        Err(e) => return Err(e.into()),
    };
    Ok(mint_response)
}
