use crate::{
    account::create_mint_deploy::create_mint_deploy,
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
    utilities::{declare_contract_v3, deploy_contract_v3},
    ConnectedAccount, ExecutionEncoding, SingleOwnerAccount,
};
use starknet_core::types::{BlockId, BlockTag, ExecutionResult, Felt, TransactionReceipt};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetClassHashAtError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),

    #[error("Account error")]
    AccountError(String),

    #[error("Unexpected receipt respose type")]
    UnexpectedReceiptResponseType,

    #[error("Unexpected execution result")]
    UnexpectedExecutionResult,

    #[error("Class hash mismatch")]
    ClassHashMismatch,
}

pub async fn get_class_hash_at(url: Url, chain_id: String) -> Result<Felt, GetClassHashAtError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        "0xb467066159b295a7667b633d6bdaabac",
    )?));

    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        Felt::from_hex("0x4d8bb41636b42d3c69039f3537333581cc19356a0c93904fa3e569498c23ad0")?,
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

    let receipt = rpc_client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await?;

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => Err(GetClassHashAtError::UnexpectedReceiptResponseType)?,
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => (),
        _ => Err(GetClassHashAtError::UnexpectedExecutionResult)?,
    };

    let class_hash_check = account
        .provider()
        .get_class_hash_at(BlockId::Tag(BlockTag::Latest), receipt.contract_address)
        .await?;

    match class_hash_check == class_hash {
        true => Ok(class_hash_check),
        false => Err(GetClassHashAtError::ClassHashMismatch),
    }
}
