use serde::de::Error;
use serde_json::Result as SerdeResult;
use serde_json::{from_reader, from_value, Value};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;
use std::fs::File;

// Function to validate JSON data for a given Txn type
pub fn validate_txn_json(file_path: &str) -> SerdeResult<()> {
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

    match txn_type {
        "INVOKE" => match txn_version {
            "0x0" => {
                let _txn: InvokeTxnV0<Felt> = from_value(value)?;
            }
            "0x1" => {
                let _txn: InvokeTxnV1<Felt> = from_value(value)?;
            }
            "0x3" => {
                let _txn: InvokeTxnV3<Felt> = from_value(value)?;
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        "DECLARE" => match txn_version {
            "0x0" => {
                let _txn: DeclareTxnV0<Felt> = from_value(value)?;
            }
            "0x1" => {
                let _txn: DeclareTxnV1<Felt> = from_value(value)?;
            }
            "0x2" => {
                let _txn: DeclareTxnV2<Felt> = from_value(value)?;
            }
            "0x3" => {
                let _txn: DeclareTxnV3<Felt> = from_value(value)?;
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        "DEPLOY_ACCOUNT" => match txn_version {
            "0x1" => {
                let _txn: DeployAccountTxnV1<Felt> = from_value(value)?;
            }
            "0x3" => {
                let _txn: DeployAccountTxnV3<Felt> = from_value(value)?;
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        _ => return Err(Error::custom("Invalid or missing transaction type")),
    }
    Ok(())
}



