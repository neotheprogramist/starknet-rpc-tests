use crate::{
    account::create_mint_deploy::create_mint_deploy,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use starknet_core::types::Transaction;
use starknet_core::types::{Felt, L1HandlerTransaction};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetTransactionByHashL1HandlerError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Unexpected tx response type")]
    UnexecpectedTxResponseType(String),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),
}

pub async fn get_transaction_by_hash_l1_handler(
    url: Url,
) -> Result<L1HandlerTransaction, GetTransactionByHashL1HandlerError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let tx = rpc_client
        .get_transaction_by_hash(Felt::from_hex(
            "0785c2ada3f53fbc66078d47715c27718f92e6e48b96372b36e5197de69b82b5",
        )?)
        .await?;

    match tx {
        Transaction::L1Handler(tx) => Ok(tx),
        _ => Err(
            GetTransactionByHashL1HandlerError::UnexecpectedTxResponseType(
                "Unexpected tx response type".to_string(),
            ),
        )?,
    }
}
