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

    let account_create_response = match create_mint_deploy(url).await {
        Ok(value) => value,
        Err(e) => return Err(GetTransactionStatusSuccededError::CreateAccountError(e)),
    };

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        account_create_response.account_data.private_key,
    ));
    println!("1");
    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        account_create_response.account_data.address,
        Felt::from_hex(&chain_id)?,
        ExecutionEncoding::New,
    );
    println!("2");
    account.set_block_id(BlockId::Tag(BlockTag::Pending));
    println!("3");
    let class_hash = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await?;
    println!("4");
    let deploy_result = deploy_contract_v3(&account, class_hash).await;
    println!("5");
    let status = rpc_client
        .get_transaction_status(deploy_result.transaction_hash)
        .await?;
    println!("6");
    match status {
        TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded) => Ok(status),
        _ => Err(GetTransactionStatusSuccededError::TransactionStatusError)?,
    }
}
