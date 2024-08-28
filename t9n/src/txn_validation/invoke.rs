use std::error::Error;

use super::constants::{DATA_AVAILABILITY_MODE_BITS, PREFIX_INVOKE};
use starknet_types_core::curve::*;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::poseidon_hash_many;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;


pub fn verify_invoke_v1_signature(txn: &InvokeTxnV1<Felt>, public_key: &str, chain_id_input: &str) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked(chain_id_input);

    let stark_key = Felt::from_hex_unchecked(public_key);

    let msg_hash = compute_hash_on_elements(&[
        PREFIX_INVOKE,
        Felt::ONE, // version
        txn.sender_address,
        Felt::ZERO, // entry_point_selector
        compute_hash_on_elements(&txn.calldata),
        txn.max_fee,
        chain_id,
        txn.nonce,
    ]);
    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
}

pub fn verify_invoke_v3_signature(txn: &InvokeTxnV3<Felt>, public_key: &str, chain_id_input: &str) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked(chain_id_input);

    let stark_key = Felt::from_hex_unchecked(public_key);

    let msg_hash = calculate_invoke_v3_transaction_hash(&chain_id, &txn).unwrap();

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
}

fn calculate_invoke_v3_transaction_hash(
    chain_id: &Felt,
    txn: &InvokeTxnV3<Felt>,
) -> Result<Felt, Box<dyn Error>> {
    let common_fields = common_fields_for_hash(PREFIX_INVOKE, *chain_id, txn)?;
    let account_deployment_data_hash = poseidon_hash_many(&txn.account_deployment_data);

    let call_data_hash = poseidon_hash_many(&txn.calldata);

    let fields_to_hash = [
        common_fields.as_slice(),
        &[account_deployment_data_hash],
        &[call_data_hash],
    ]
    .concat();

    let txn_hash = poseidon_hash_many(fields_to_hash.as_slice());
    Ok(txn_hash)
}

/// Returns the array of Felts that reflects (tip, resource_bounds_for_fee) from SNIP-8
fn get_resource_bounds_array(
    txn: &InvokeTxnV3<Felt>,
) -> Result<Vec<Felt>, Box<dyn Error>> {
    let mut array = Vec::<Felt>::new();
    array.push(txn.tip);

    array.push(field_element_from_resource_bounds(
        Resource::L1Gas,
        &txn.resource_bounds.l1_gas,
    )?);
    array.push(field_element_from_resource_bounds(
        Resource::L2Gas,
        &txn.resource_bounds.l2_gas,
    )?);

    Ok(array)
}

fn field_element_from_resource_bounds(
    resource: Resource,
    resource_bounds: &ResourceBounds,
) -> Result<Felt, Box<dyn Error>> {
    let resource_name_as_json_string = serde_json::to_value(resource)?;

    // Ensure it's a string and get bytes
    let resource_name_bytes = resource_name_as_json_string
        .as_str()
        .ok_or("Resource name is not a string")?
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
    txn: &InvokeTxnV3<Felt>,
) -> Result<Vec<Felt>, Box<dyn Error>> {
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
fn get_data_availability_modes_field_element(txn: &InvokeTxnV3<Felt>) -> Felt {
    let da_mode = get_data_availability_mode_value_as_u64(txn.nonce_data_availability_mode.clone())
        << DATA_AVAILABILITY_MODE_BITS;
    let da_mode =
        da_mode + get_data_availability_mode_value_as_u64(txn.fee_data_availability_mode.clone());
    Felt::from(da_mode)
}
