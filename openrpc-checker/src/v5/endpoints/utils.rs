use reqwest::Client;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::{Pedersen, StarkHash};
use starknet_types_rpc::v0_5_0::{ContractClass, TxnHash};
use tokio::io::AsyncReadExt;
use tracing::{debug, error};
use url::Url;

use crate::v5::{
    accounts::account::{normalize_address, starknet_keccak},
    contract::{CompiledClass, HashAndFlatten, SierraClass},
    endpoints::errors::RpcError,
};

use super::{declare_contract::RunnerError, errors::NonAsciiNameError};

const DEFAULT_ENTRY_POINT_NAME: &str = "__default__";
const DEFAULT_L1_ENTRY_POINT_NAME: &str = "__l1_default__";

pub async fn get_compiled_contract(
    sierra_path: &str,
    casm_path: &str,
) -> Result<(ContractClass, TxnHash), RunnerError> {
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

pub async fn restart_devnet(url: Url) -> Result<(), RpcError> {
    let client = Client::new();
    let url = url.join("/restart")?;
    let response = client.post(url).send().await?;
    if response.status().is_success() {
        debug!("Devnet restarted successfully.");
        Ok(())
    } else {
        error!("Failed to restart Devnet. Status: {}", response.status());
        Err(RpcError::RequestError(
            response.error_for_status().unwrap_err(),
        ))
    }
}

pub fn get_selector_from_name(func_name: &str) -> Result<Felt, NonAsciiNameError> {
    if func_name == DEFAULT_ENTRY_POINT_NAME || func_name == DEFAULT_L1_ENTRY_POINT_NAME {
        Ok(Felt::ZERO)
    } else {
        let name_bytes = func_name.as_bytes();
        if name_bytes.is_ascii() {
            Ok(starknet_keccak(name_bytes))
        } else {
            Err(NonAsciiNameError)
        }
    }
}
#[allow(dead_code)]
pub fn get_storage_var_address(var_name: &str, args: &[Felt]) -> Result<Felt, NonAsciiNameError> {
    let var_name_bytes = var_name.as_bytes();
    if var_name_bytes.is_ascii() {
        let mut res = starknet_keccak(var_name_bytes);
        for arg in args.iter() {
            res = Pedersen::hash(&res, arg);
        }
        Ok(normalize_address(res))
    } else {
        Err(NonAsciiNameError)
    }
}
