use super::constants::{ADDR_BOUND, PREFIX_CONTRACT_ADDRESS, PREFIX_DEPLOY_ACCOUNT};
use starknet_types_core::curve::*;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{v0_7_1::starknet_api_openrpc::*, DeployAccountTxn};

pub fn verify_deploy_account_signature(txn: DeployAccountTxn<Felt>) -> Result<bool, VerifyError> {
    match txn {
        DeployAccountTxn::V1(deploy_account_txn) => {
            verify_deploy_account_v1_signature(&deploy_account_txn)
        }

        DeployAccountTxn::V3(deploy_account_txn) => {
            verify_deploy_account_v3_signature(&deploy_account_txn)
        }
    }
}

fn verify_deploy_account_v1_signature(txn: &DeployAccountTxnV1<Felt>) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked("0x534e5f5345504f4c4941");

    let stark_key = Felt::from_hex_unchecked(
        "0x917acc840787b45091ce55cd50c82e65cb0a7344cf18fd6d337f3a54f8b70c",
    );

    let mut calldata_to_hash: Vec<Felt> = Vec::new();
    calldata_to_hash.push(txn.class_hash);
    calldata_to_hash.push(txn.contract_address_salt);
    calldata_to_hash.extend(txn.constructor_calldata.clone());

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
    println!(
        "{:?}",
        calculate_contract_address(
            txn.contract_address_salt,
            txn.class_hash,
            compute_hash_on_elements(&txn.constructor_calldata)
        )
    );
    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
}

fn verify_deploy_account_v3_signature(txn: &DeployAccountTxnV3<Felt>) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked("0x534e5f5345504f4c4941");

    let stark_key = Felt::from_hex_unchecked(
        "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b",
    );

    let mut calldata_to_hash: Vec<Felt> = Vec::new();
    calldata_to_hash.push(txn.class_hash);
    calldata_to_hash.push(txn.contract_address_salt);
    calldata_to_hash.extend(txn.constructor_calldata.clone());

    let msg_hash = compute_hash_on_elements(&[
        PREFIX_DEPLOY_ACCOUNT,
        Felt::THREE, // version
        calculate_contract_address(
            txn.contract_address_salt,
            txn.class_hash,
            compute_hash_on_elements(&txn.constructor_calldata),
        ),
        // TODO: compute_hash_on_elements(&[txn.tip, l1_gas_bounds, l2_gas_bounds]),
        compute_hash_on_elements(&txn.paymaster_data),
        chain_id,
        txn.nonce,
        //TODO: data_availability_modes,
        compute_hash_on_elements(&txn.constructor_calldata),
        txn.class_hash,
        txn.contract_address_salt,
    ]);
    println!(
        "{:?}",
        calculate_contract_address(
            txn.contract_address_salt,
            txn.class_hash,
            compute_hash_on_elements(&txn.constructor_calldata)
        )
    );
    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
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
