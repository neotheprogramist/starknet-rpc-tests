use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use starknet_core::types::{BlockId, BlockTag, BlockWithTxs, MaybePendingBlockWithTxs};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetBlockWithTxsError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Unexpected block type")]
    UnexpectedBlockResponseType(String),
}

pub async fn get_block_with_txs(url: Url) -> Result<BlockWithTxs, GetBlockWithTxsError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url));

    let block = rpc_client
        .get_block_with_txs(BlockId::Tag(BlockTag::Latest))
        .await?;

    match block {
        MaybePendingBlockWithTxs::Block(block) => Ok(block),
        _ => Err(GetBlockWithTxsError::UnexpectedBlockResponseType(
            "Unexpected block response type".to_string(),
        ))?,
    }
}
