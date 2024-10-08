use super::constants::{DATA_AVAILABILITY_MODE_BITS, PREFIX_CONTRACT_CLASS_V0_1_0, PREFIX_DECLARE};
use super::errors::Error;
use crypto_utils::curve::signer::{compute_hash_on_elements, verify};
use sha3::{Digest, Keccak256};
use starknet_types_core::felt::{Felt, NonZeroFelt};
use starknet_types_core::hash::poseidon_hash::{poseidon_hash_many, PoseidonHasher};
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;
use starknet_types_rpc::v0_7_1::SierraEntryPoint;

// 2 ** 251 - 256
const ADDR_BOUND: NonZeroFelt = NonZeroFelt::from_raw([
    576459263475590224,
    18446744073709255680,
    160989183,
    18446743986131443745,
]);

pub fn verify_declare_v2_signature(
    txn: &BroadcastedDeclareTxnV2<Felt>,
    public_key: &str,
    chain_id_input: &str,
) -> Result<(bool, Felt), Error> {
    let chain_id = Felt::from_hex_unchecked(chain_id_input);
    let stark_key = Felt::from_hex_unchecked(public_key);

    let msg_hash = compute_hash_on_elements(&[
        PREFIX_DECLARE,
        Felt::TWO, // version
        txn.sender_address,
        Felt::ZERO, // entry_point_selector
        compute_hash_on_elements(&[class_hash(txn.contract_class.clone())]),
        txn.max_fee,
        chain_id,
        txn.nonce,
        txn.compiled_class_hash,
    ]);

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    match verify(&stark_key, &msg_hash, &r_bytes, &s_bytes) {
        Ok(is_valid) => Ok((is_valid, msg_hash)),
        Err(e) => Err(Error::VerifyError(e)),
    }
}

pub fn verify_declare_v3_signature(
    txn: &BroadcastedDeclareTxnV3<Felt>,
    public_key: &str,
    chain_id_input: &str,
) -> Result<(bool, Felt), Error> {
    let chain_id = Felt::from_hex_unchecked(chain_id_input);
    let stark_key = Felt::from_hex_unchecked(public_key);

    let class_hash = class_hash(txn.contract_class.clone());
    let msg_hash = calculate_transaction_v3_hash(&chain_id, txn, class_hash)?;

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    match verify(&stark_key, &msg_hash, &r_bytes, &s_bytes) {
        Ok(is_valid) => Ok((is_valid, msg_hash)),
        Err(e) => Err(Error::VerifyError(e)),
    }
}

fn class_hash(contract_class: ContractClass<Felt>) -> Felt {
    let mut hasher = PoseidonHasher::new();
    hasher.update(PREFIX_CONTRACT_CLASS_V0_1_0);
    hasher.update(hash_entrypoints(
        &contract_class.entry_points_by_type.external,
    ));
    hasher.update(hash_entrypoints(
        &contract_class.entry_points_by_type.l1_handler,
    ));
    hasher.update(hash_entrypoints(
        &contract_class.entry_points_by_type.constructor,
    ));
    hasher.update(starknet_keccak(
        contract_class.abi.clone().expect("abi expected").as_bytes(),
    ));
    hasher.update(poseidon_hash_many(&contract_class.sierra_program));

    normalize_address(hasher.finalize())
}

fn normalize_address(address: Felt) -> Felt {
    address.mod_floor(&ADDR_BOUND)
}

fn hash_entrypoints(entrypoints: &[SierraEntryPoint<Felt>]) -> Felt {
    let mut hasher = PoseidonHasher::new();
    for entry in entrypoints.iter() {
        hasher.update(entry.selector);
        hasher.update(entry.function_idx.into());
    }
    hasher.finalize()
}

fn starknet_keccak(data: &[u8]) -> Felt {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let mut hash = hasher.finalize();

    // Remove the first 6 bits
    hash[0] &= 0b00000011;

    // Because we know hash is always 32 bytes
    Felt::from_bytes_be(unsafe { &*(hash[..].as_ptr() as *const [u8; 32]) })
}

fn calculate_transaction_v3_hash(
    chain_id: &Felt,
    txn: &BroadcastedDeclareTxnV3<Felt>,
    class_hash: Felt,
) -> Result<Felt, Error> {
    let common_fields = common_fields_for_hash(PREFIX_DECLARE, *chain_id, txn)?;
    let account_deployment_data_hash = poseidon_hash_many(&txn.account_deployment_data);

    let fields_to_hash = [
        common_fields.as_slice(),
        &[account_deployment_data_hash],
        &[class_hash],
        &[txn.compiled_class_hash],
    ]
    .concat();

    let txn_hash = poseidon_hash_many(fields_to_hash.as_slice());
    Ok(txn_hash)
}

/// Returns the array of Felts that reflects (tip, resource_bounds_for_fee) from SNIP-8
fn get_resource_bounds_array(txn: &BroadcastedDeclareTxnV3<Felt>) -> Result<Vec<Felt>, Error> {
    Ok(vec![
        txn.tip,
        field_element_from_resource_bounds(Resource::L1Gas, &txn.resource_bounds.l1_gas)?,
        field_element_from_resource_bounds(Resource::L2Gas, &txn.resource_bounds.l2_gas)?,
    ])
}

fn field_element_from_resource_bounds(
    resource: Resource,
    resource_bounds: &ResourceBounds,
) -> Result<Felt, Error> {
    let resource_name_as_json_string = serde_json::to_value(resource)?;

    // Ensure it's a string and get bytes
    let resource_name_bytes = resource_name_as_json_string
        .as_str()
        .ok_or(Error::ResourceNameError)?
        .as_bytes();

    let max_amount_hex_str = resource_bounds.max_amount.as_str().trim_start_matches("0x");
    let max_amount_u64 = u64::from_str_radix(max_amount_hex_str, 16)?;

    let max_price_per_unit_hex_str = resource_bounds
        .max_price_per_unit
        .as_str()
        .trim_start_matches("0x");
    let max_price_per_unit_u64 = u128::from_str_radix(max_price_per_unit_hex_str, 16)?;

    // (resource||max_amount||max_price_per_unit) from SNIP-8 https://github.com/starknet-io/SNIPs/blob/main/SNIPS/snip-8.md#protocol-changes
    let bytes: Vec<u8> = [
        resource_name_bytes,
        max_amount_u64.to_be_bytes().as_slice(),
        max_price_per_unit_u64.to_be_bytes().as_slice(),
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();

    Ok(Felt::from_bytes_be_slice(&bytes))
}

fn common_fields_for_hash(
    tx_prefix: Felt,
    chain_id: Felt,
    txn: &BroadcastedDeclareTxnV3<Felt>,
) -> Result<Vec<Felt>, Error> {
    let array: Vec<Felt> = vec![
        tx_prefix,                                                      // TX_PREFIX
        Felt::THREE,                                                    // version
        txn.sender_address,                                             // address
        poseidon_hash_many(get_resource_bounds_array(txn)?.as_slice()), /* h(tip, resource_bounds_for_fee) */
        poseidon_hash_many(&txn.paymaster_data),                        // h(paymaster_data)
        chain_id,                                                       // chain_id
        txn.nonce,                                                      // nonce
        get_data_availability_modes_field_element(txn), /* nonce_data_availability ||
                                                         * fee_data_availability_mode */
    ];

    Ok(array)
}

fn get_data_availability_mode_value_as_u64(data_availability_mode: DaMode) -> u64 {
    match data_availability_mode {
        DaMode::L1 => 0,
        DaMode::L2 => 1,
    }
}

/// Returns Felt that encodes the data availability modes of the transaction
fn get_data_availability_modes_field_element(txn: &BroadcastedDeclareTxnV3<Felt>) -> Felt {
    let da_mode = get_data_availability_mode_value_as_u64(txn.nonce_data_availability_mode.clone())
        << DATA_AVAILABILITY_MODE_BITS;
    let da_mode =
        da_mode + get_data_availability_mode_value_as_u64(txn.fee_data_availability_mode.clone());
    Felt::from(da_mode)
}
