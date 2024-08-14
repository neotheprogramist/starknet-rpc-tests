use crate::txn_validation::declare::*;
use crate::txn_validation::deploy_account::*;
use crate::txn_validation::invoke::*;
use serde::de::Error;
use serde_json::Result as SerdeResult;
use serde_json::{from_reader, from_value, Value};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;
use starknet_types_rpc::{
    BroadcastedDeclareTxn, BroadcastedDeployAccountTxn, BroadcastedInvokeTxn,
};
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

    let trimmed_version = txn_version.trim_start_matches("0x").trim_start_matches("0");

    let formatted_version = format!("0x{}", trimmed_version);

    let version = formatted_version.as_str();

    match txn_type {
        "INVOKE" => match version {
            "0x0" => {
                let txn: InvokeTxnV0<Felt> = from_value(value)?;
                println!(
                    "{:?}",
                    verify_invoke_signature(&BroadcastedInvokeTxn::V0(txn))
                );
            }
            "0x1" => {
                let txn: InvokeTxnV1<Felt> = from_value(value)?;
                println!(
                    "{:?}",
                    verify_invoke_signature(&BroadcastedInvokeTxn::V1(txn))
                );
            }
            "0x3" => {
                let txn: InvokeTxnV3<Felt> = from_value(value)?;
                println!(
                    "{:?}",
                    verify_invoke_signature(&BroadcastedInvokeTxn::V3(txn))
                );
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        "DECLARE" => match version {
            "0x1" => {
                let txn: BroadcastedDeclareTxnV1<Felt> = from_value(value)?;
                println!(
                    "{:?}",
                    verify_declare_signature(&BroadcastedDeclareTxn::V1(txn))
                );
            }
            "0x2" => {
                let txn: BroadcastedDeclareTxnV2<Felt> = from_value(value)?;
                println!(
                    "{:?}",
                    verify_declare_signature(&BroadcastedDeclareTxn::V2(txn))
                );
            }
            "0x3" => {
                let txn: BroadcastedDeclareTxnV3<Felt> = from_value(value)?;
                println!(
                    "{:?}",
                    verify_declare_signature(&BroadcastedDeclareTxn::V3(txn))
                );
            }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        "DEPLOY_ACCOUNT" => match version {
            "0x1" => {
                let txn: DeployAccountTxnV1<Felt> = from_value(value)?;
                println!(
                    "{:?}",
                    verify_deploy_account_signature(BroadcastedDeployAccountTxn::V1(txn))
                );
            }
            // TODO: V3 in BroadcastedDeployAccountTxn not available "0x3" => {
            //     let txn: DeployAccountTxnV3<Felt> = from_value(value)?;
            //     BroadcastedDeployAccountTxn::V3(txn);

            // }
            _ => return Err(serde_json::Error::custom("Unsupported version")),
        },
        _ => return Err(Error::custom("Invalid or missing transaction type")),
    }
    Ok(())
}
