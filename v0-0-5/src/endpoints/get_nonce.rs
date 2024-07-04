use crate::{
    account::create_mint_deploy::create_mint_deploy,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::ProviderError,
    ConnectedAccount, ExecutionEncoding, SingleOwnerAccount,
};
use starknet_core::types::{BlockId, BlockTag, Felt};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetNonceError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),
}

pub struct GetNonceResponse {
    pub nonce: Felt,
}

pub async fn get_nonce(url: Url, chain_id: String) -> Result<GetNonceResponse, GetNonceError> {
    let account_create_response = match create_mint_deploy(url.clone()).await {
        Ok(value) => value,
        Err(e) => return Err(GetNonceError::CreateAccountError(e)),
    };

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        account_create_response.account_data.private_key,
    ));

    let mut accout = SingleOwnerAccount::new(
        JsonRpcClient::new(HttpTransport::new(url)),
        signer,
        account_create_response.account_data.address,
        Felt::from_hex(&chain_id)?,
        ExecutionEncoding::New,
    );

    accout.set_block_id(BlockId::Tag(BlockTag::Pending));
    let nonce = accout.get_nonce().await?;

    Ok(GetNonceResponse { nonce })
}
