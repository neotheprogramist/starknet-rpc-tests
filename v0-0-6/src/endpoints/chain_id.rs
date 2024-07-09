use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};

use colored::*;
use starknet_core::types::Felt;
use thiserror::Error;
use tracing::info;
use url::Url;

#[derive(Error, Debug)]
pub enum ChainIdError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),
}

pub async fn get_chain_id(url: Url) -> Result<Felt, ChainIdError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let chain_id = rpc_client.chain_id().await?;
    info!("{}", "Get Chain Id Compatible".green());
    Ok(chain_id)
}
