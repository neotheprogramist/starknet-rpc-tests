use std::sync::Arc;

use crate::v7::rpc::accounts::account::{Account, AccountError};
use crate::v7::rpc::contract::HashAndFlatten;
use crate::v7::rpc::providers::provider::ProviderError;
use crate::v7::rpc::{
    accounts::single_owner::SingleOwnerAccount,
    contract::{CompiledClass, SierraClass},
    providers::provider::Provider,
    signers::local_wallet::LocalWallet,
};

use regex::Regex;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::{ContractClass, TxnHash};

use thiserror::Error;
use tokio::io::AsyncReadExt;
use tracing::info;
use url::ParseError;

#[allow(dead_code)]
pub async fn declare_contract<P: Provider + Send + Sync>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    sierra_path: &str,
    casm_path: &str,
) -> Result<Felt, RunnerError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            info!("General account error encountered: {:?}, possible cause - incorrect address or public_key in environment variables!", e);
            Err(RunnerError::AccountFailure(format!("Account error: {}", e)))
        }
    }
}

pub fn parse_class_hash_from_error(error_msg: &str) -> Felt {
    info!("Error message: {}", error_msg);
    let re = Regex::new(r#"StarkFelt\("(0x[a-fA-F0-9]+)"\)"#).unwrap();

    // Attempt to capture the class hash
    if let Some(captures) = re.captures(error_msg) {
        if let Some(contract_address) = captures.get(1) {
            return Felt::from_hex(contract_address.as_str()).expect("Failed to parse class hash");
        }
    }

    panic!("Failed to extract class hash from error message");
}

pub fn extract_class_hash_from_error(error_msg: &str) -> Option<Felt> {
    let re = Regex::new(r#"0x[a-fA-F0-9]{64}"#).unwrap();

    if let Some(capture) = re.find(error_msg) {
        return Some(Felt::from_hex_unchecked(capture.as_str()));
    }

    None
}

#[allow(dead_code)]
pub async fn get_compiled_contract(
    sierra_path: &str,
    casm_path: &str,
) -> Result<(ContractClass<Felt>, TxnHash<Felt>), RunnerError> {
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

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum RunnerError {
    #[error(transparent)]
    ParsingError(#[from] ParseError),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("ReadFileError error: {0}")]
    ReadFileError(String),

    #[error("Account error: {0}")]
    AccountFailure(String),

    #[error("Deployment error: {0}")]
    DeploymentFailure(String),

    #[error(transparent)]
    BoxError(#[from] Box<dyn std::error::Error>),

    #[error("Starknet-devnet not launched : {0}")]
    DevnetNotLaunched(String),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
