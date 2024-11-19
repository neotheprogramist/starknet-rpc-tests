use crate::StateMachine;
use crate::StateMachineResult;
use flate2::read::GzDecoder;
use production_nodes_types::pathfinder_types::types::gateway_add_transaction::*;
use production_nodes_types::pathfinder_types::types::gateway_state_update::StarknetVersion;
use serde::Deserialize;
use serde_json::Result as JsonResult;
use serde_json::{json, Value};
use serde_with::{serde_as, DisplayFromStr};
use starknet_types_core::felt::Felt;
use starknet_types_core::felt::FromStrError;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use t9n::txn_validation::validate::validate_txn_json;
use thiserror::Error;

const SN_SEPOLIA: &str = "0x534e5f5345504f4c4941";

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),

    #[error("Failed to decompress Gzip data: {0}")]
    GzipDecompressionError(#[from] std::io::Error),

    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SierraDecodeError(#[from] DecodeError),

    #[error("Validation succeeded but no hash found")]
    MissingHash,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error(transparent)]
    StrToFeltError(#[from] FromStrError),
}

#[derive(Clone)]
pub struct Ok;

#[derive(Clone)]
pub struct Invalid;

#[derive(Clone)]
pub struct Skipped;

#[derive(Clone)]
pub struct AddTransactionStateMachine<S> {
    path: String,
    request_type: Option<AddTransactionRequestType>,
    transaction_type: Option<AddTransactionResponseType>,
    state: S,
}

impl<S> AddTransactionStateMachine<S> {
    pub fn get_state(&self) -> &S {
        &self.state
    }
}

impl AddTransactionStateMachine<Ok> {
    pub fn new() -> Self {
        Self {
            path: "/gateway/add_transaction".to_string(),
            state: Ok,
            request_type: None,
            transaction_type: None,
        }
    }

    pub fn to_invalid(self) -> AddTransactionStateMachine<Invalid> {
        AddTransactionStateMachine {
            path: self.path,
            state: Invalid,
            request_type: self.request_type,
            transaction_type: self.transaction_type,
        }
    }

    pub fn to_skipped(&self) -> AddTransactionStateMachine<Skipped> {
        AddTransactionStateMachine {
            path: self.path.clone(),
            state: Skipped,
            request_type: None,
            transaction_type: None,
        }
    }
}

impl Default for AddTransactionStateMachine<Ok> {
    fn default() -> Self {
        Self::new()
    }
}

impl AddTransactionStateMachine<Invalid> {
    pub fn to_skipped(self) -> AddTransactionStateMachine<Skipped> {
        AddTransactionStateMachine {
            path: self.path,
            state: Skipped,
            request_type: None,
            transaction_type: None,
        }
    }
}

pub enum AddTransactionStateMachineWrapper {
    Ok(AddTransactionStateMachine<Ok>),
    Invalid(AddTransactionStateMachine<Invalid>),
    Skipped(AddTransactionStateMachine<Skipped>),
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Default)]
struct CheckVersion {
    block: Version,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Default)]
struct Version {
    #[serde_as(as = "DisplayFromStr")]
    starknet_version: StarknetVersion,
}

pub fn validate_request(request: String) -> Result<String, ValidationError> {
    let mut json_value: Value =
        serde_json::from_str(&request).map_err(ValidationError::JsonParseError)?;

    let txn_type = json_value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ValidationError::ValidationError("Missing or invalid 'type' field".to_string())
        })?;

    let txn_version = json_value
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ValidationError::ValidationError("Missing or invalid 'version' field".to_string())
        })?;

    if txn_type == "DEPLOY_ACCOUNT" && txn_version == "0x1" {
        // Convert constructor_calldata to hex
        if let Some(constructor_calldata) = json_value.get_mut("constructor_calldata") {
            if let Some(calldata_array) = constructor_calldata.as_array_mut() {
                // Convert each decimal string to a hexadecimal string
                for value in calldata_array.iter_mut() {
                    if let Some(dec_str) = value.as_str() {
                        *value = json!(Felt::from_dec_str(dec_str)?.to_hex_string());
                    }
                }
            }
        }
        // Convert signature to hex
        if let Some(signature) = json_value.get_mut("signature") {
            if let Some(signature_array) = signature.as_array_mut() {
                for value in signature_array.iter_mut() {
                    if let Some(dec_str) = value.as_str() {
                        *value = json!(Felt::from_dec_str(dec_str)?.to_hex_string());
                    }
                }
            }
        }
    }

    if let Some(type_field) = json_value.get_mut("type") {
        if type_field == "INVOKE_FUNCTION" {
            *type_field = json!("INVOKE");
        }
    }

    // nonce_data_availability_mode from 0 to L1
    if let Some(nonce_mode) = json_value.get_mut("nonce_data_availability_mode") {
        *nonce_mode = if nonce_mode == &json!(0) {
            json!("L1")
        } else {
            json!("L2")
        };
    }

    // fee_data_availability_mode from 0 to L1
    if let Some(fee_mode) = json_value.get_mut("fee_data_availability_mode") {
        *fee_mode = if fee_mode == &json!(0) {
            json!("L1")
        } else {
            json!("L2")
        };
    }

    if let Some(resource_bounds) = json_value.get_mut("resource_bounds") {
        if let Some(resource_bounds_map) = resource_bounds.as_object_mut() {
            if let Some(l1_gas) = resource_bounds_map.remove("L1_GAS") {
                resource_bounds_map.insert("l1_gas".to_string(), l1_gas);
            }
            if let Some(l2_gas) = resource_bounds_map.remove("L2_GAS") {
                resource_bounds_map.insert("l2_gas".to_string(), l2_gas);
            }
        }
    }

    if let Some(contract_class) = json_value.get_mut("contract_class") {
        if let Some(sierra_program) = contract_class.get_mut("sierra_program") {
            let encoded_data = sierra_program.as_str().ok_or_else(|| {
                ValidationError::ValidationError("sierra_program must be a string".to_string())
            })?;
            let decoded_array = decode_declare_sierra(encoded_data.to_string())
                .map_err(ValidationError::SierraDecodeError)?;
            *sierra_program = json!(decoded_array);
        }
    }

    let modified_json_str =
        serde_json::to_string_pretty(&json_value).map_err(ValidationError::JsonParseError)?;

    let target_dir = Path::new("target/shared");
    fs::create_dir_all(target_dir).map_err(ValidationError::IoError)?;

    let file_path = target_dir.join("request_txn.json");
    File::create(&file_path)
        .and_then(|mut file| file.write_all(modified_json_str.as_bytes()))
        .map_err(ValidationError::IoError)?;

    let path = PathBuf::from("target/shared/request_txn.json");

    match validate_txn_json(&path, None, SN_SEPOLIA) {
        Result::Ok(json_result) => {
            if let Some(hash) = json_result.get("hash").and_then(|v| v.as_str()) {
                Result::Ok(hash.to_string())
            } else {
                Err(ValidationError::MissingHash)
            }
        }
        Err(e) => Err(ValidationError::ValidationError(format!(
            "Validation error: {}",
            e
        ))),
    }
}

fn decode_declare_sierra(declare_txn: String) -> Result<Vec<Felt>, DecodeError> {
    let decoded_data = base64::decode(declare_txn)?;
    let mut gz_decoder = GzDecoder::new(decoded_data.as_slice());

    let mut decompressed_data = Vec::new();
    gz_decoder.read_to_end(&mut decompressed_data)?;

    let result = decompressed_data;

    let decoded_and_unzipped = result;
    let sierra_program: Vec<Felt> = serde_json::from_slice(&decoded_and_unzipped)?;
    Result::Ok(sierra_program)
}

impl StateMachine for AddTransactionStateMachineWrapper {
    fn run(
        &mut self,
        request_body: String,
        response_body: String,
        path: String,
    ) -> StateMachineResult {
        if self.filter(path) {
            self.step(request_body, response_body)
        } else {
            *self = AddTransactionStateMachineWrapper::Skipped(match self {
                AddTransactionStateMachineWrapper::Ok(machine) => machine.to_skipped(),
                AddTransactionStateMachineWrapper::Invalid(machine) => machine.clone().to_skipped(),
                AddTransactionStateMachineWrapper::Skipped(machine) => machine.clone(),
            });
            StateMachineResult::Skipped
        }
    }

    fn filter(&self, path: String) -> bool {
        match self {
            AddTransactionStateMachineWrapper::Ok(machine) => machine.path == path,
            AddTransactionStateMachineWrapper::Invalid(machine) => machine.path == path,
            AddTransactionStateMachineWrapper::Skipped(machine) => machine.path == path,
        }
    }
    fn step(&mut self, request_body: String, response_body: String) -> StateMachineResult {
        *self = match self {
            AddTransactionStateMachineWrapper::Ok(machine) => {
                match serde_json::from_str::<AddTransactionRequestType>(&request_body) {
                    JsonResult::Ok(version) => {
                        machine.request_type = Some(version);
                        AddTransactionStateMachineWrapper::Ok(machine.clone())
                    }
                    Err(_) => {
                        AddTransactionStateMachineWrapper::Invalid(machine.clone().to_invalid())
                    }
                }
            }
            AddTransactionStateMachineWrapper::Invalid(machine) => {
                AddTransactionStateMachineWrapper::Invalid(machine.clone())
            }
            AddTransactionStateMachineWrapper::Skipped(machine) => {
                AddTransactionStateMachineWrapper::Skipped(machine.clone())
            }
        };

        *self = match self {
            AddTransactionStateMachineWrapper::Ok(machine) => match machine.request_type.clone() {
                Some(AddTransactionRequestType::Invoke) => {
                    match serde_json::from_str::<InvokeResponse>(&response_body) {
                        JsonResult::Ok(invoke_response) => {
                            let valid_tx = match validate_request(request_body) {
                                Result::Ok(hash) => {
                                    Felt::from_hex_unchecked(&hash)
                                        == invoke_response.transaction_hash
                                }
                                Err(_) => false,
                            };
                            if valid_tx {
                                machine.transaction_type =
                                    Some(AddTransactionResponseType::Invoke(invoke_response));
                                AddTransactionStateMachineWrapper::Ok(machine.clone())
                            } else {
                                machine.transaction_type = None;
                                AddTransactionStateMachineWrapper::Ok(machine.clone())
                            }
                        }
                        Err(_) => {
                            AddTransactionStateMachineWrapper::Invalid(machine.clone().to_invalid())
                        }
                    }
                }
                Some(AddTransactionRequestType::Declare) => {
                    match serde_json::from_str::<DeclareResponse>(&response_body) {
                        JsonResult::Ok(delcare_response) => {
                            let valid_tx = match validate_request(request_body) {
                                Result::Ok(hash) => {
                                    Felt::from_hex_unchecked(&hash)
                                        == delcare_response.transaction_hash
                                }
                                Err(_) => false,
                            };
                            if valid_tx {
                                machine.transaction_type =
                                    Some(AddTransactionResponseType::Declare(delcare_response));

                                AddTransactionStateMachineWrapper::Ok(machine.clone())
                            } else {
                                machine.transaction_type = None;
                                AddTransactionStateMachineWrapper::Ok(machine.clone())
                            }
                        }
                        Err(_) => {
                            AddTransactionStateMachineWrapper::Invalid(machine.clone().to_invalid())
                        }
                    }
                }
                Some(AddTransactionRequestType::DeployAccount) => {
                    match serde_json::from_str::<DeployAccountResponse>(&response_body) {
                        JsonResult::Ok(deploy_acc_response) => {
                            let valid_tx = match validate_request(request_body) {
                                Result::Ok(hash) => {
                                    Felt::from_hex_unchecked(&hash)
                                        == deploy_acc_response.transaction_hash
                                }
                                Err(_) => false,
                            };
                            if valid_tx {
                                machine.transaction_type = Some(
                                    AddTransactionResponseType::DeployAccount(deploy_acc_response),
                                );
                                AddTransactionStateMachineWrapper::Ok(machine.clone())
                            } else {
                                machine.transaction_type = None;
                                AddTransactionStateMachineWrapper::Ok(machine.clone())
                            }
                        }
                        Err(_) => {
                            AddTransactionStateMachineWrapper::Invalid(machine.clone().to_invalid())
                        }
                    }
                }
                None => AddTransactionStateMachineWrapper::Invalid(machine.clone().to_invalid()),
            },
            AddTransactionStateMachineWrapper::Invalid(machine) => {
                AddTransactionStateMachineWrapper::Invalid(machine.clone())
            }
            AddTransactionStateMachineWrapper::Skipped(machine) => {
                AddTransactionStateMachineWrapper::Skipped(machine.clone())
            }
        };

        match self {
            AddTransactionStateMachineWrapper::Ok(_) => {
                let message = self.get_message();
                StateMachineResult::Ok(message)
            }
            AddTransactionStateMachineWrapper::Invalid(_) => {
                let message = self.get_message();
                StateMachineResult::Invalid(message)
            }
            AddTransactionStateMachineWrapper::Skipped(_) => StateMachineResult::Skipped,
        }
    }
}

impl AddTransactionStateMachineWrapper {
    pub fn new() -> Self {
        AddTransactionStateMachineWrapper::Ok(AddTransactionStateMachine::new())
    }

    pub fn get_message(&self) -> String {
        match self {
            AddTransactionStateMachineWrapper::Ok(machine) => machine
                .transaction_type
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "Transaction type not set".to_string()),
            AddTransactionStateMachineWrapper::Invalid(machine) => machine
                .transaction_type
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "Transaction type not set".to_string()),
            AddTransactionStateMachineWrapper::Skipped(_) => "".to_string(),
        }
    }
}

impl Default for AddTransactionStateMachineWrapper {
    fn default() -> Self {
        Self::new()
    }
}
