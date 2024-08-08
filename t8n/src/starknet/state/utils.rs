use blockifier::versioned_constants::VersionedConstants;
use serde_json::Value;
use starknet_devnet_types::{
    felt::Felt,
    patricia_key::{PatriciaKey, StorageKey},
};
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

/// Returns the storage address of a Starknet storage variable given its name and arguments.
pub(crate) fn get_storage_var_address(
    storage_var_name: &str,
    args: &[Felt],
) -> DevnetResult<StorageKey> {
    let storage_var_address = starknet_rs_core::utils::get_storage_var_address(
        storage_var_name,
        &args
            .iter()
            .map(|f| FieldElement::from(*f))
            .collect::<Vec<FieldElement>>(),
    )
    .map_err(|err| Error::UnexpectedInternalError {
        msg: err.to_string(),
    })?;

    Ok(PatriciaKey::new(Felt::new(
        storage_var_address.to_bytes_be(),
    )?)?)
}

pub(crate) fn get_versioned_constants() -> VersionedConstants {
    VersionedConstants::create_for_testing()
}

pub mod random_number_generator {
    use rand::{thread_rng, Rng, SeedableRng};
    use rand_mt::Mt64;

    pub fn generate_u32_random_number() -> u32 {
        thread_rng().gen()
    }

    pub(crate) fn generate_u128_random_numbers(seed: u32, random_numbers_count: u8) -> Vec<u128> {
        let mut result: Vec<u128> = Vec::new();
        let mut rng: Mt64 = SeedableRng::seed_from_u64(seed as u64);

        for _ in 0..random_numbers_count {
            result.push(rng.gen());
        }

        result
    }
}
