use super::errors::Error;
use crate::txn_hashes::deploy_account::{
    calculate_deploy_account_v1_hash, calculate_deploy_v3_transaction_hash,
};
use crypto_utils::curve::signer::verify;
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

    let msg_hash = calculate_deploy_account_v1_hash(txn, &chain_id)?;

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

    let msg_hash = calculate_deploy_v3_transaction_hash(txn, &chain_id)?;

    let r_bytes = txn.signature[0];
    let s_bytes = txn.signature[1];

    match verify(&stark_key, &msg_hash, &r_bytes, &s_bytes) {
        Ok(is_valid) => Ok((is_valid, msg_hash)),
        Err(e) => Err(Error::VerifyError(e)),
    }
}
