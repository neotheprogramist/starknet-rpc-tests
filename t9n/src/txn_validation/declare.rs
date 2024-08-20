use sha3::{Digest, Keccak256};
use starknet_types_core::curve::*;
use starknet_types_core::{
    felt::{Felt, NonZeroFelt},
    hash::{poseidon_hash_many, PoseidonHasher},
};
use starknet_types_rpc::v0_7_1::SierraEntryPoint;
use starknet_types_rpc::{v0_7_1::starknet_api_openrpc::*, BroadcastedDeclareTxn};
use super::constants::{PREFIX_CONTRACT_CLASS_V0_1_0, PREFIX_DECLARE};

// 2 ** 251 - 256
const ADDR_BOUND: NonZeroFelt = NonZeroFelt::from_raw([
    576459263475590224,
    18446744073709255680,
    160989183,
    18446743986131443745,
]);

pub fn verify_declare_signature(txn: &BroadcastedDeclareTxn<Felt>) -> Result<bool, VerifyError> {
    // checking version
    match txn {
        BroadcastedDeclareTxn::V1(declare_txn) => verify_declare_v1_signature(declare_txn),
        BroadcastedDeclareTxn::V2(declare_txn) => verify_declare_v2_signature(declare_txn),
        BroadcastedDeclareTxn::V3(declare_txn) => verify_declare_v3_signature(declare_txn),
        BroadcastedDeclareTxn::QueryV1(_) => todo!(),
        BroadcastedDeclareTxn::QueryV2(_) => todo!(),
        BroadcastedDeclareTxn::QueryV3(_) => todo!(),
    }
}

fn verify_declare_v1_signature(txn: &BroadcastedDeclareTxnV1<Felt>) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked("0x534e5f5345504f4c4941");

    let stark_key = Felt::from_hex_unchecked(
        "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b",
    );

    let msg_hash = compute_hash_on_elements(&[
        PREFIX_DECLARE,
        Felt::ONE, // version
        txn.sender_address,
        Felt::ZERO, // entry_point_selector
        compute_hash_on_elements(&[class_hash(txn.contract_class.clone())]),
        txn.max_fee,
        chain_id,
        txn.nonce,
    ]);

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
}

fn verify_declare_v2_signature(txn: &BroadcastedDeclareTxnV2<Felt>) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked("0x534e5f5345504f4c4941");

    let stark_key = Felt::from_hex_unchecked(
        "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b",
    );

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

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
}

fn verify_declare_v3_signature(txn: &BroadcastedDeclareTxnV3<Felt>) -> Result<bool, VerifyError> {
    let chain_id = Felt::from_hex_unchecked("0x534e5f5345504f4c4941");

    let stark_key = Felt::from_hex_unchecked(
        "0x39d9e6ce352ad4530a0ef5d5a18fd3303c3606a7fa6ac5b620020ad681cc33b",
    );

    let msg_hash = compute_hash_on_elements(&[
        PREFIX_DECLARE,
        Felt::THREE, // version
        txn.sender_address,
        // compute_hash_on_elements(&[txn.tip, l1_gas_bounds, l2_gas_bounds]),
        compute_hash_on_elements(&txn.paymaster_data),
        chain_id,
        txn.nonce,
        // data availability modes
        compute_hash_on_elements(&txn.account_deployment_data),
        compute_hash_on_elements(&[class_hash(txn.contract_class.clone())]),
        txn.compiled_class_hash,
    ]);

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    verify(&stark_key, &msg_hash, &r_bytes, &s_bytes)
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
