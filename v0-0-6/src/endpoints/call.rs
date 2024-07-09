use crate::{
    account::create_mint_deploy::create_mint_deploy,
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
    utilities::{declare_contract_v3, deploy_contract_v3},
    ExecutionEncoding, SingleOwnerAccount,
};
use colored::*;
use starknet_core::{
    types::{BlockId, BlockTag, ExecutionResult, Felt, FunctionCall, TransactionReceipt},
    utils::get_selector_from_name,
};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use tracing::info;
use url::Url;

#[derive(Error, Debug)]
pub enum CallError {
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

pub async fn call(url: Url, chain_id: String) -> Result<Vec<Felt>, CallError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    // let account_create_response = match create_mint_deploy(url).await {
    //     Ok(value) => value,
    //     Err(e) => return Err(CallError::CreateAccountError(e)),
    // };

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        // account_create_response.account_data.private_key,
        Felt::from_hex("0xe1406455b7d66b1690803be066cbe5e")?,
    ));

    let account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        Felt::from_hex("0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1")?,
        // account_create_response.account_data.address,
        Felt::from_hex(&chain_id)?,
        ExecutionEncoding::New,
    );
    info!("TEST1");
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
    info!("TEST2");
    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => Err(CallError::UnexpectedReceiptType)?,
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => Err(CallError::UnexpectedExecutionResult)?,
    }
    info!("TEST3");
    let eth_balance = rpc_client
        .call(
            &FunctionCall {
                contract_address: receipt.contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;
    info!("{}", "Call compatible".green());
    Ok(eth_balance)
}
