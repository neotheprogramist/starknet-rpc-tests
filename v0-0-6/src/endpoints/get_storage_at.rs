use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use colored::*;

use starknet_core::{types::Felt, utils::NonAsciiNameError};
use starknet_core::{
    types::{BlockId, BlockTag},
    utils::get_storage_var_address,
};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use tracing::info;
use url::Url;
#[derive(Error, Debug)]
pub enum BlockNumberError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Non ascii name error")]
    NonAsciiNameError(#[from] NonAsciiNameError),
}

pub async fn get_storage_at(url: Url) -> Result<Felt, BlockNumberError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    // Checks L2 ETH balance via storage taking advantage of implementation detail
    let eth_balance = rpc_client
        .get_storage_at(
            Felt::from_hex("049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")?,
            get_storage_var_address(
                "ERC20_balances",
                &[Felt::from_hex(
                    "03f47d3911396b6d579fd7848cf576286ab6f96dda977915d6c7b10f3dd2315b",
                )?],
            )?,
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;
    info!("{}", "Get Storage At Compatible".green());
    Ok(eth_balance)
}
