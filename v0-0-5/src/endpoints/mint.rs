use rand::Rng;
use reqwest::{Client, StatusCode};
use serde::de::{self};
use serde::{Deserialize, Serialize};
use starknet_core::types::{BlockTag, Felt, U256};
use starknet_signers::LocalWallet;
use thiserror::Error;
use url::Url;

use crate::jsonrpc::{HttpTransport, JsonRpcClient};
use crate::provider::Provider;
use crate::{Account, ConnectedAccount, SingleOwnerAccount};
mod u256_mint {
    use super::*;
    use crypto_bigint::U256 as CryptoBigintU256;
    use num::bigint::{BigInt, Sign};
    use serde::de::Deserialize;
    use serde::{Deserializer, Serializer};
    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        println!("VALUE {:?}", value);
        let string = value.to_string();
        println!("STRING {:?}", string);
        serializer.serialize_str(&string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        println!("STRING {:?}", s);
        let bigint = s.parse::<BigInt>().map_err(de::Error::custom)?;
        println!("BIGINT {:?}", bigint);
        let be_bytes = bigint.to_bytes_be().1;

        // Check if the bigint is negative or larger than U256 max value
        if bigint.sign() == Sign::Minus || bigint.bits() > 256 {
            return Err(de::Error::custom("Value out of range for U256"));
        }

        let expected_length = 32;
        let mut be_bytes = if be_bytes.len() < expected_length {
            let padding = vec![0u8; expected_length - be_bytes.len()];
            [padding, be_bytes].concat()
        } else {
            be_bytes[be_bytes.len() - expected_length..].to_vec()
        };

        let crypto_bigint = CryptoBigintU256::from_be_slice(&be_bytes);
        let data = U256::from(crypto_bigint);
        println!("DATA {:?}", data);
        Ok(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintRequest {
    pub amount: u128,
    pub address: Felt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintResponse {
    #[serde(with = "u256_mint")]
    new_balance: U256,
    unit: FeeUnit,
    tx_hash: Felt,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FeeUnit {
    WEI,
    FRI,
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

    #[error("Url Join Error")]
    JoinUrlError(#[from] url::ParseError),
}

pub async fn mint(base_url: Url, mint_request: &MintRequest) -> Result<MintResponse, MintError> {
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

async fn fuzzy_test_mint(
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let test_count = rng.gen_range(2..=5);

    for _ in 0..test_count {
        let initial_balance = account
            .provider()
            .get_account_balance(account.address(), FeeUnit::WEI, BlockTag::Latest)
            .await?;

        let mint_amount = rng.gen_range(u128::MIN + 1..=u128::MAX);

        let mint_result = account
            .provider()
            .mint(account.address(), mint_amount)
            .await?;

        let new_balance = account
            .provider()
            .get_account_balance(account.address(), FeeUnit::WEI, BlockTag::Latest)
            .await?;
    }

    Ok(())
}
