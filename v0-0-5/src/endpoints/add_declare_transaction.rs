use crate::{
    account::create_mint_deploy::create_mint_deploy,
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::ProviderError,
    utilities::declare_contract_v3,
    ExecutionEncoding, SingleOwnerAccount,
};
use starknet_core::types::Felt;
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum AddDeclareTransactionError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),
}

pub async fn add_declare_transaction(
    url: Url,
    chain_id: String,
) -> Result<Felt, AddDeclareTransactionError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let account_create_response = match create_mint_deploy(url).await {
        Ok(value) => value,
        Err(e) => return Err(AddDeclareTransactionError::CreateAccountError(e)),
    };

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        account_create_response.account_data.private_key,
    ));

    let account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        account_create_response.account_data.address,
        Felt::from_hex(&chain_id)?,
        ExecutionEncoding::New,
    );

    let response = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await?;
    Ok(response)
}
