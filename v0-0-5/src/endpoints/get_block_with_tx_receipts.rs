use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use starknet_core::types::{BlockId, BlockTag, BlockWithReceipts, MaybePendingBlockWithReceipts};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetBlockWithTxReceiptsError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Unexpected block type")]
    UnexpectedBlockResponseType(String),
}

pub struct GetBlockWithTxReceiptsResponse {
    pub block: BlockWithReceipts,
}

pub async fn get_block_with_receipts(
    url: Url,
) -> Result<GetBlockWithTxReceiptsResponse, GetBlockWithTxReceiptsError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url));

    let block = rpc_client
        .get_block_with_receipts(BlockId::Tag(BlockTag::Latest))
        .await?;
    println!("{:?}", block);

    let response = match block {
        MaybePendingBlockWithReceipts::Block(block) => GetBlockWithTxReceiptsResponse { block },
        _ => Err(GetBlockWithTxReceiptsError::UnexpectedBlockResponseType(
            "Unexpected block response type".to_string(),
        ))?,
    };

    Ok(response)
}
