use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use starknet_core::types::{BlockId, BlockTag};

use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetBlockTransactionCountError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),
}

pub async fn get_block_transaction_count(url: Url) -> Result<u64, GetBlockTransactionCountError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let count = rpc_client
        .get_block_transaction_count(BlockId::Tag(BlockTag::Latest))
        .await?;

    Ok(count)
}
