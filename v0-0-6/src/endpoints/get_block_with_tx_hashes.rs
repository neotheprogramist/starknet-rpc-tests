use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use colored::*;
use starknet_core::types::{BlockId, BlockTag, BlockWithTxHashes, MaybePendingBlockWithTxHashes};
use thiserror::Error;
use tracing::info;
use url::Url;

#[derive(Error, Debug)]
pub enum GetBlockWithTxHashesError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Unexpected block type")]
    UnexpectedBlockResponseType(String),
}

pub struct GetBlockWithTxHashesResponse {
    pub block: BlockWithTxHashes,
}

pub async fn get_block_with_tx_hashes(
    url: Url,
) -> Result<GetBlockWithTxHashesResponse, GetBlockWithTxHashesError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url));

    let block = rpc_client
        .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
        .await?;

    let response = match block {
        MaybePendingBlockWithTxHashes::Block(block) => GetBlockWithTxHashesResponse { block },
        _ => Err(GetBlockWithTxHashesError::UnexpectedBlockResponseType(
            "Unexpected block response type".to_string(),
        ))?,
    };
    info!("{}", "Get Block With Tx Hashes Compatible".green());
    Ok(response)
}
