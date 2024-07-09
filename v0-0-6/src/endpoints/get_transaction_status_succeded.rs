use crate::{
    account::create_mint_deploy::create_mint_deploy,
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
    utilities::{declare_contract_v3, deploy_contract_v3},
    ExecutionEncoding, SingleOwnerAccount,
};
use starknet_core::types::{
    BlockId, BlockTag, Felt, TransactionExecutionStatus, TransactionStatus,
};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetTransactionStatusSuccededError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),

    #[error("Unexpected transaction status")]
    TransactionStatusError,
}

pub async fn get_transaction_status_succeeded(
    url: Url,
    chain_id: String,
) -> Result<TransactionStatus, GetTransactionStatusSuccededError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    // let account_create_response = match create_mint_deploy(url).await {
    //     Ok(value) => value,
    //     Err(e) => return Err(GetTransactionStatusSuccededError::CreateAccountError(e)),
    // };

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        "0x71d7bb07b9a64f6f78ac4c816aff4da9",
    )?));

    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        Felt::from_hex("0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691")?,
        Felt::from_hex(&chain_id)?,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let class_hash = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await?;

    let deploy_result = deploy_contract_v3(&account, class_hash).await;

    let status = rpc_client
        .get_transaction_status(deploy_result.transaction_hash)
        .await?;

    match status {
        TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded) => Ok(status),
        _ => Err(GetTransactionStatusSuccededError::TransactionStatusError)?,
    }
}
