use serde_json::Value;
use starknet_rs_core::types::{contract::CompiledClass, FieldElement};

use super::errors::{DevnetResult, Error};

/// Returns the hash of a compiled class.
/// # Arguments
/// * `casm_json` - The compiled class in JSON format.
pub fn casm_hash(casm_json: Value) -> DevnetResult<FieldElement> {
    serde_json::from_value::<CompiledClass>(casm_json)
        .map_err(|err| Error::DeserializationError {
            origin: err.to_string(),
        })?
        .class_hash()
        .map_err(|err| Error::UnexpectedInternalError {
            msg: err.to_string(),
        })
}
