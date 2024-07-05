use std::sync::Arc;

use crate::contract::factory::ContractFactory;
use crate::errors::errors::{parse_class_hash_from_error, RunnerError};
use crate::jsonrpc::HttpTransport;
use crate::provider::{Provider, ProviderError};
use crate::{jsonrpc::JsonRpcClient, ExecutionEncoding, SingleOwnerAccount};
use crate::{Account, AccountError};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use starknet_core::types::contract::{CompiledClass, SierraClass};
use starknet_core::types::{
    BlockId, BlockTag, ExecutionResult, FlattenedSierraClass, InvokeTransactionResult,
};
use starknet_core::types::{Felt, TransactionReceipt};
use starknet_signers::{LocalWallet, SigningKey};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use url::{ParseError, Url};

pub async fn decalare_and_deploy(
    sender_address: Felt,
    private_key: Felt,
    chain_id: Felt,
    sierra_path: &str,
    casm_path: &str,
    url: Url,
) -> (
    SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    Felt,
) {
    let client = JsonRpcClient::new(HttpTransport::new(url));
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));
    let mut account = SingleOwnerAccount::new(
        client.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let class_hash = declare_contract_v3(&account, sierra_path, casm_path)
        .await
        .unwrap();
    let deploy_result = deploy_contract_v3(&account, class_hash).await;
    let receipt = client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await
        .unwrap();
    assert!(receipt.block.is_block());

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => panic!("unexpected receipt response type"),
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => panic!("unexpected execution result"),
    }
    (account, receipt.contract_address)
}

pub async fn declare_contract_v3<P: Provider + Send + Sync>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    sierra_path: &str,
    casm_path: &str,
) -> Result<Felt, RunnerError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    match account
        .declare_v3(Arc::new(flattened_sierra_class), compiled_class_hash)
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                )))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
            } else {
                Err(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    starkneterror
                )))
            }
        }
        Err(e) => {
            tracing::info!("General account error encountered: {:?}, possible cause - incorrect address or public_key in environment variables!", e);
            Err(RunnerError::AccountFailure(format!("Account error: {}", e)))
        }
    }
}

pub async fn deploy_contract_v3<P: Provider + Send + Sync>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    class_hash: Felt,
) -> InvokeTransactionResult {
    let factory = ContractFactory::new(class_hash, account);
    let mut salt_buffer = [0u8; 32];
    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut salt_buffer[1..]);
    let result = factory
        .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
        .send()
        .await
        .unwrap();
    result
}

pub async fn get_compiled_contract(
    sierra_path: &str,
    casm_path: &str,
) -> Result<(FlattenedSierraClass, Felt), RunnerError> {
    let mut file = tokio::fs::File::open(sierra_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut sierra = String::default();
    file.read_to_string(&mut sierra)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let mut file = tokio::fs::File::open(casm_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut casm = String::default();
    file.read_to_string(&mut casm)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let contract_artifact: SierraClass = serde_json::from_str(&sierra)?;
    let compiled_class: CompiledClass = serde_json::from_str(&casm)?;
    let casm_class_hash = compiled_class.class_hash().unwrap();
    let flattened_class = contract_artifact.clone().flatten().unwrap();
    Ok((flattened_class, casm_class_hash))
}
