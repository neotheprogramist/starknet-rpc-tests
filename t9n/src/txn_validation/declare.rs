use super::errors::Error;
use crate::txn_hashes::declare_hash::{calculate_declare_v2_hash, calculate_declare_v3_hash};
use crypto_utils::curve::signer::verify;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;

pub fn verify_declare_v2_signature(
    txn: &BroadcastedDeclareTxnV2<Felt>,
    public_key: &str,
    chain_id_input: &str,
) -> Result<(bool, Felt), Error> {
    let chain_id = Felt::from_hex_unchecked(chain_id_input);
    let stark_key = Felt::from_hex_unchecked(public_key);

    let msg_hash = calculate_declare_v2_hash(txn, &chain_id)?;
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

    let msg_hash = calculate_declare_v3_hash(txn, &chain_id)?;

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    match verify(&stark_key, &msg_hash, &r_bytes, &s_bytes) {
        Ok(is_valid) => Ok((is_valid, msg_hash)),
        Err(e) => Err(Error::VerifyError(e)),
    }
}
