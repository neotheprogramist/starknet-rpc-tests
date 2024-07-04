use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use starknet_core::types::{
    BlockId, BlockTag, BlockWithTxHashes, Felt, MaybePendingBlockWithTxHashes,
};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetBlockWithTxHashesError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Unexpected block type")]
    UnexpectedResult(String),
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
        _ => Err(GetBlockWithTxHashesError::UnexpectedResult(
            "Unexpected block response type".to_string(),
        ))?,
    };
    Ok(response)
}
