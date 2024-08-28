use crate::txn_validation::declare::*;
use crate::txn_validation::deploy_account::*;
use crate::txn_validation::invoke::*;
use serde::de::Error;
use serde_json::Result as SerdeResult;
use serde_json::{from_reader, from_value, Value};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;
use starknet_types_rpc::DeployAccountTxn;
use std::fs::File;

pub fn validate_txn_json(file_path: &str, public_key: &str, chain_id: &str) -> SerdeResult<()> {
    
    let file = File::open(file_path).map_err(|e| serde_json::Error::custom(e.to_string()))?;

    let value: Value = from_reader(file)?;

    let txn_type = value
        .get("type")
        .ok_or_else(|| serde_json::Error::missing_field("type"))?
        .as_str()
        .ok_or_else(|| serde_json::Error::custom("Invalid type format"))?;
    let txn_version = value
        .get("version")
        .ok_or_else(|| serde_json::Error::missing_field("version"))?
        .as_str()
        .ok_or_else(|| serde_json::Error::custom("Invalid version format"))?;

    let trimmed_version = txn_version.trim_start_matches("0x").trim_start_matches("0");

    let formatted_version = format!("0x{}", trimmed_version);

    let version = formatted_version.as_str();

    match txn_type {
        "INVOKE" => match version {
            "0x1" => {
                let txn: InvokeTxnV1<Felt> = from_value(value)?;
                match verify_invoke_v1_signature(&txn, public_key, chain_id) {
                    Ok(true) => println!("Signature is valid"),
                    Ok(false) => return Err(serde_json::Error::custom("Signature is invalid")),
                    Err(e) => println!("Verification error: {:?}", e),
                }
            }
            "0x3" => {
                let txn: InvokeTxnV3<Felt> = from_value(value)?;
                match verify_invoke_v3_signature(&txn, public_key, chain_id) {
                    Ok(true) => println!("Signature is valid"),
                    Ok(false) => return Err(serde_json::Error::custom("Signature is invalid")),
                    Err(e) => println!("Verification error: {:?}", e),
                }
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        "DECLARE" => match version {
            "0x2" => {
                let txn: BroadcastedDeclareTxnV2<Felt> = from_value(value)?;
                match verify_declare_v2_signature(&txn, public_key, chain_id) {
                    Ok(true) => println!("Signature is valid"),
                    Ok(false) => return Err(serde_json::Error::custom("Signature is invalid")),
                    Err(e) => println!("Verification error: {:?}", e),
                }
            }
            "0x3" => {
                let txn: BroadcastedDeclareTxnV3<Felt> = from_value(value)?;
                match verify_declare_v3_signature(&txn, public_key, chain_id) {
                    Ok(true) => println!("Signature is valid"),
                    Ok(false) => return Err(serde_json::Error::custom("Signature is invalid")),
                    Err(e) => println!("Verification error: {:?}", e),
                }
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        "DEPLOY_ACCOUNT" => match version {
            "0x1" => {
                let txn: DeployAccountTxnV1<Felt> = from_value(value)?;
                match verify_deploy_account_signature(DeployAccountTxn::V1(txn), public_key, chain_id) {
                    Ok(true) => println!("Signature is valid"),
                    Ok(false) => return Err(serde_json::Error::custom("Signature is invalid")),
                    Err(e) => println!("Verification error: {:?}", e),
                }
            }
            "0x3" => {
                let txn: DeployAccountTxnV3<Felt> = from_value(value)?;
                match verify_deploy_account_signature(DeployAccountTxn::V3(txn), public_key, chain_id) {
                    Ok(true) => println!("Signature is valid"),
                    Ok(false) => return Err(serde_json::Error::custom("Signature is invalid")),
                    Err(e) => println!("Verification error: {:?}", e),
                }
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        _ => return Err(Error::custom("Invalid or missing transaction type")),
    }
    Ok(())
}
