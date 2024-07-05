use crate::{
    account::create_mint_deploy::create_mint_deploy,
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
    utilities::{declare_contract_v3, deploy_contract_v3},
    ExecutionEncoding, SingleOwnerAccount,
};

use starknet_core::types::{DeployTransactionReceipt, ExecutionResult, Felt, TransactionReceipt};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetTranasactionReceiptDeployError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),

    #[error("Unexpected receipt response type")]
    UnexpectedReceiptType,

    #[error("Unexpected execution result")]
    UnexpectedExecutionResult,
}

pub async fn get_transaction_receipt_deploy(
    url: Url,
    chain_id: String,
) -> Result<DeployTransactionReceipt, GetTranasactionReceiptDeployError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let account_create_response = match create_mint_deploy(url).await {
        Ok(value) => value,
        Err(e) => return Err(GetTranasactionReceiptDeployError::CreateAccountError(e)),
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

    let class_hash = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await?;

    let deploy_result = deploy_contract_v3(&account, class_hash).await;

    let receipt = rpc_client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => Err(GetTranasactionReceiptDeployError::UnexpectedReceiptType)?,
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => Err(GetTranasactionReceiptDeployError::UnexpectedExecutionResult)?,
    }

    Ok(receipt)
}
