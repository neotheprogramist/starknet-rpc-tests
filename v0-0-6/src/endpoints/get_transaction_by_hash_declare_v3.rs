use std::sync::Arc;

use crate::{
    account::create_mint_deploy::create_mint_deploy,
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
    utilities::get_compiled_contract,
    Account, ExecutionEncoding, SingleOwnerAccount,
};
use colored::*;
use starknet_core::types::{
    BlockId, BlockTag, DeclareTransaction, DeclareTransactionV3, Felt, Transaction,
};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use tracing::info;
use url::Url;

#[derive(Error, Debug)]
pub enum GetTransactionByHashDeclareV3Error {
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

    #[error("Unexpected tx response type")]
    UnexpectedTxResponseType,
}

pub async fn get_transaction_by_hash_declare_v3(
    url: Url,
    chain_id: String,
) -> Result<DeclareTransactionV3, GetTransactionByHashDeclareV3Error> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        "0xe1406455b7d66b1690803be066cbe5e",
    )?));

    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        Felt::from_hex("0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1")?,
        Felt::from_hex(&chain_id)?,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
        "../target/dev/sample_SampleStarknet.contract_class.json",
        "../target/dev/sample_SampleStarknet.compiled_contract_class.json",
    )
    .await?;

    let declare_result = account
        .declare_v3(Arc::new(flattened_sierra_class), compiled_class_hash)
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
        .map_err(|err| GetTransactionByHashDeclareV3Error::AccountError(err.to_string()))?;

    let tx = rpc_client
        .get_transaction_by_hash(declare_result.transaction_hash)
        .await?;

    match tx {
        Transaction::Declare(DeclareTransaction::V3(tx)) => {
            info!(
                "{}",
                "Get Transaction By Hash Declare V3 Compatible".green()
            );
            Ok(tx)
        }
        _ => Err(GetTransactionByHashDeclareV3Error::UnexpectedTxResponseType)?,
    }
}
