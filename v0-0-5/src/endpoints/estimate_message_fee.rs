use crate::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
};
use starknet_core::types::{
    eth_address::FromHexError, BlockId, BlockTag, EthAddress, FeeEstimate, Felt, MsgFromL1,
};

use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum EstimateMessageFeeError {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("EthAddress from hex error")]
    EthAddressFromHexError(#[from] FromHexError),

    #[error("Felt from hex error")]
    FeltFromHexError(#[from] FromStrError),
}

pub async fn estimate_message_fee(url: Url) -> Result<FeeEstimate, EstimateMessageFeeError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let estimate = rpc_client
        .estimate_message_fee(
            MsgFromL1 {
                from_address: EthAddress::from_hex("0x8453FC6Cd1bCfE8D4dFC069C400B433054d47bDc")?,
                to_address: Felt::from_hex(
                    "04c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f",
                )?,
                entry_point_selector: Felt::from_hex(
                    "02d757788a8d8d6f21d1cd40bce38a8222d70654214e96ff95d8086e684fbee5",
                )?,
                payload: vec![Felt::ONE, Felt::ONE, Felt::ONE],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;

    Ok(estimate)
}
