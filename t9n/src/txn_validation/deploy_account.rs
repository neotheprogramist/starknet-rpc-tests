use super::errors::Error;

use super::constants::{
    ADDR_BOUND, DATA_AVAILABILITY_MODE_BITS, PREFIX_CONTRACT_ADDRESS, PREFIX_DEPLOY_ACCOUNT,
};
use crypto_utils::curve::signer::{compute_hash_on_elements, verify};
use starknet_types_core::hash::poseidon_hash::poseidon_hash_many;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{v0_7_1::starknet_api_openrpc::*, DeployAccountTxn};

pub fn verify_deploy_account_signature(
    txn: DeployAccountTxn<Felt>,
    public_key: &str,
    chain_id_input: &str,
) -> Result<(bool, Felt), Error> {
    match txn {
        DeployAccountTxn::V1(deploy_account_txn) => {
            verify_deploy_account_v1_signature(&deploy_account_txn, public_key, chain_id_input)
        }

        DeployAccountTxn::V3(deploy_account_txn) => {
            verify_deploy_account_v3_signature(&deploy_account_txn, public_key, chain_id_input)
        }
    }
}

fn verify_deploy_account_v1_signature(
    txn: &DeployAccountTxnV1<Felt>,
    public_key: &str,
    chain_id_input: &str,
) -> Result<(bool, Felt), Error> {
    let chain_id = Felt::from_hex_unchecked(chain_id_input);
    let stark_key = Felt::from_hex_unchecked(public_key);

    let mut calldata_to_hash = vec![txn.class_hash, txn.contract_address_salt];
    calldata_to_hash.extend(txn.constructor_calldata.iter());

    let msg_hash = compute_hash_on_elements(&[
        PREFIX_DEPLOY_ACCOUNT,
        Felt::ONE, // version
        calculate_contract_address(
            txn.contract_address_salt,
            txn.class_hash,
            compute_hash_on_elements(&txn.constructor_calldata),
        ),
        Felt::ZERO, // entry_point_selector
        compute_hash_on_elements(&calldata_to_hash),
        txn.max_fee,
        chain_id,
        txn.nonce,
    ]);

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    match verify(&stark_key, &msg_hash, &r_bytes, &s_bytes) {
        Ok(is_valid) => Ok((is_valid, msg_hash)),
        Err(e) => Err(Error::VerifyError(e)),
    }
}

fn verify_deploy_account_v3_signature(
    txn: &DeployAccountTxnV3<Felt>,
    public_key: &str,
    chain_id_input: &str,
) -> Result<(bool, Felt), Error> {
    let chain_id = Felt::from_hex_unchecked(chain_id_input);
    let stark_key = Felt::from_hex_unchecked(public_key);

    let msg_hash = calculate_deploy_v3_transaction_hash(&chain_id, &txn)?;

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    match verify(&stark_key, &msg_hash, &r_bytes, &s_bytes) {
        Ok(is_valid) => Ok((is_valid, msg_hash)),
        Err(e) => Err(Error::VerifyError(e)),
    }
}

fn calculate_contract_address(
    salt: Felt,
    class_hash: Felt,
    constructor_calldata_hash: Felt,
) -> Felt {
    compute_hash_on_elements(&[
        PREFIX_CONTRACT_ADDRESS,
        Felt::ZERO,
        salt,
        class_hash,
        constructor_calldata_hash,
    ])
    .mod_floor(&ADDR_BOUND)
}

fn calculate_deploy_v3_transaction_hash(
    chain_id: &Felt,
    txn: &DeployAccountTxnV3<Felt>,
) -> Result<Felt, Error> {
    let common_fields = common_fields_for_hash(PREFIX_DEPLOY_ACCOUNT, *chain_id, txn)?;

    let constructor_calldata_hash = poseidon_hash_many(&txn.constructor_calldata);

    let fields_to_hash = [
        common_fields.as_slice(),
        &[constructor_calldata_hash],
        &[txn.class_hash],
        &[txn.contract_address_salt],
    ]
    .concat();

    let txn_hash = poseidon_hash_many(fields_to_hash.as_slice());
    Ok(txn_hash)
}

/// Returns the array of Felts that reflects (tip, resource_bounds_for_fee) from SNIP-8
fn get_resource_bounds_array(txn: &DeployAccountTxnV3<Felt>) -> Result<Vec<Felt>, Error> {
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
    txn: &DeployAccountTxnV3<Felt>,
) -> Result<Vec<Felt>, Error> {
    let array: Vec<Felt> = vec![
        tx_prefix,   // TX_PREFIX
        Felt::THREE, // version
        calculate_contract_address(
            txn.contract_address_salt,
            txn.class_hash,
            compute_hash_on_elements(&txn.constructor_calldata.clone()),
        ),
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
fn get_data_availability_modes_field_element(txn: &DeployAccountTxnV3<Felt>) -> Felt {
    let da_mode = get_data_availability_mode_value_as_u64(txn.nonce_data_availability_mode.clone())
        << DATA_AVAILABILITY_MODE_BITS;
    let da_mode =
        da_mode + get_data_availability_mode_value_as_u64(txn.fee_data_availability_mode.clone());
    Felt::from(da_mode)
}
