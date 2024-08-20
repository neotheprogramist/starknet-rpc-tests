use super::constants::PREFIX_INVOKE;
use starknet_types_core::curve::*;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;
use starknet_types_rpc::BroadcastedInvokeTxn;

pub fn verify_invoke_signature(txn: &BroadcastedInvokeTxn<Felt>) -> Result<bool, VerifyError> {
    match txn {
        BroadcastedInvokeTxn::V0(invoke_txn) => {
            // Handle V1 specific signature verification
            // TODO: verify_v1_signature(declare_txn))
            println!("{:?}", invoke_txn);
            Ok(true)
        }
        BroadcastedInvokeTxn::V1(invoke_txn) => verify_invoke_v1_signature(&invoke_txn),
        BroadcastedInvokeTxn::V3(invoke_txn) => verify_invoke_v3_signature(&invoke_txn),
        BroadcastedInvokeTxn::QueryV0(_) => todo!(),
        BroadcastedInvokeTxn::QueryV1(_) => todo!(),
        BroadcastedInvokeTxn::QueryV3(_) => todo!(),
    }
}

fn verify_invoke_v1_signature(txn: &InvokeTxnV1<Felt>) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked("0x534e5f5345504f4c4941");

    let stark_key = Felt::from_hex_unchecked(
        "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b",
    );

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

fn verify_invoke_v3_signature(txn: &InvokeTxnV3<Felt>) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked("0x534e5f5345504f4c4941");

    let stark_key = Felt::from_hex_unchecked(
        "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b",
    );
    // Create the buffer for L1 resources and compute its hash
    let mut resource_buffer_l1 = [
        0, 0, b'L', b'1', b'_', b'G', b'A', b'S', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    // resource_buffer_l1[8..(8 + 8)]
    //     .copy_from_slice(txn.resource_bounds.l1_gas.max_amount.as_bytes());
    // resource_buffer_l1[(8 + 8)..]
    //     .copy_from_slice(txn.resource_bounds.l1_gas.max_price_per_unit.as_bytes());
    // let l1_gas_bounds = Felt::from_bytes_be(&resource_buffer_l1);

    // Create the buffer for L2 resources and compute its hash
    let resource_buffer_l2 = [
        0, 0, b'L', b'2', b'_', b'G', b'A', b'S', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let l2_gas_bounds = Felt::from_bytes_be(&resource_buffer_l2);

    let msg_hash = compute_hash_on_elements(&[
        PREFIX_INVOKE,
        Felt::THREE, // version
        txn.nonce,
        txn.sender_address,
        // compute_hash_on_elements(&[Felt::ZERO, l1_gas_bounds, l2_gas_bounds]),
        compute_hash_on_elements(&txn.paymaster_data),
        chain_id,
        //TODO :data_availability_modes,
        compute_hash_on_elements(&txn.account_deployment_data),
        compute_hash_on_elements(&txn.calldata),
    ]);
    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
}
