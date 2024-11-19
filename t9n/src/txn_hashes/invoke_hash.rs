use super::constants::{DATA_AVAILABILITY_MODE_BITS, PREFIX_INVOKE};
use crate::txn_validation::errors::Error;
use crypto_utils::curve::signer::compute_hash_on_elements;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::{Poseidon, StarkHash};
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;

pub fn calculate_invoke_v1_hash(txn: &InvokeTxnV1<Felt>, chain_id: &Felt) -> Result<Felt, Error> {
    Ok(compute_hash_on_elements(&[
        PREFIX_INVOKE,
        Felt::ONE, // version
        txn.sender_address,
        Felt::ZERO, // entry_point_selector
        compute_hash_on_elements(&txn.calldata),
        txn.max_fee,
        *chain_id,
        txn.nonce,
    ]))
}

pub fn calculate_invoke_v3_hash(txn: &InvokeTxnV3<Felt>, chain_id: &Felt) -> Result<Felt, Error> {
    let common_fields = common_fields_for_hash(PREFIX_INVOKE, *chain_id, txn)?;
    let account_deployment_data_hash = Poseidon::hash_array(&txn.account_deployment_data);

    let call_data_hash = Poseidon::hash_array(&txn.calldata);

    let fields_to_hash = [
        common_fields.as_slice(),
        &[account_deployment_data_hash],
        &[call_data_hash],
    ]
    .concat();

    Ok(Poseidon::hash_array(&fields_to_hash))
}

/// Returns the array of Felts that reflects (tip, resource_bounds_for_fee) from SNIP-8
/// Returns the array of Felts that reflects (tip, resource_bounds_for_fee) from SNIP-8
fn get_resource_bounds_array(txn: &InvokeTxnV3<Felt>) -> Result<Vec<Felt>, Error> {
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
    txn: &InvokeTxnV3<Felt>,
) -> Result<Vec<Felt>, Error> {
    let array: Vec<Felt> = vec![
        tx_prefix,                                                        // TX_PREFIX
        Felt::THREE,                                                      // version
        txn.sender_address,                                               // address
        Poseidon::hash_array(get_resource_bounds_array(txn)?.as_slice()), /* h(tip, resource_bounds_for_fee) */
        Poseidon::hash_array(&txn.paymaster_data),                        // h(paymaster_data)
        chain_id,                                                         // chain_id
        txn.nonce,                                                        // nonce
        get_data_availability_modes_field_element(txn), /* nonce_data_availability ||  fee_data_availability_mode */
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
