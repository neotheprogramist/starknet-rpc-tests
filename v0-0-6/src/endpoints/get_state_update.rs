use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use colored::*;
use starknet_core::types::{BlockId, BlockTag, MaybePendingStateUpdate, StateUpdate};
use thiserror::Error;
use tracing::info;
use url::Url;
#[derive(Error, Debug)]
pub enum GetStateUpdateError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Unexpected block type")]
    UnexpectedDataType(String),
}

pub async fn get_state_update(url: Url) -> Result<StateUpdate, GetStateUpdateError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url));

    let state_update = rpc_client
        .get_state_update(BlockId::Tag(BlockTag::Latest))
        .await?;

    let response = match state_update {
        MaybePendingStateUpdate::Update(value) => {
            info!("{}", "Get State Update Compatible".green());
            value
        }
        _ => Err(GetStateUpdateError::UnexpectedDataType(
            "Unexpected data type".to_string(),
        ))?,
    };

    Ok(response)
}
