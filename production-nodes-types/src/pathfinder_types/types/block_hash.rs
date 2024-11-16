use std::io;

use super::block::BlockHeaderData;
use super::block_builder_input::TransactionHash;
use super::header::{BlockHeader, L1DataAvailabilityMode};
use crate::pathfinder_types::types::event::Event;
use crate::pathfinder_types::types::hash::{FeltHash, PoseidonHash};
use crate::pathfinder_types::types::receipt::ExecutionStatus;
use anyhow::{Context, Result};
use crypto_utils::hash::{poseidon_hash_many, PoseidonHasher};
// use starknet_types_core::hash::{poseidon_hash_many, PoseidonHasher};
// use starknet_types_core::hash::{poseidon_hash_many, PoseidonHasher};
use starknet_types_core::hash::{Poseidon, StarkHash};
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::{
    DeclareTxn, DeployAccountTxn, InvokeTxn, Txn, TxnWithHash,
};

use super::transaction::TransactionOrEventTree;
use sha3::Digest;
use starknet_types_core::felt::Felt;

impl BlockHeaderData {
    pub fn from_header(header: &BlockHeader) -> Self {
        Self {
            hash: header.hash,
            parent_hash: header.parent_hash,
            number: header.number,
            timestamp: header.timestamp,
            sequencer_address: header.sequencer_address,
            state_commitment: header.state_commitment,
            transaction_commitment: header.transaction_commitment,
            transaction_count: header
                .transaction_count
                .try_into()
                .expect("ptr size is 64bits"),
            event_commitment: header.event_commitment,
            event_count: header.event_count.try_into().expect("ptr size is 64bits"),
            starknet_version: header.starknet_version.to_string(),
            state_diff_length: header.state_diff_length,
            eth_l1_gas_price: header.eth_l1_gas_price,
            strk_l1_gas_price: header.strk_l1_gas_price,
            eth_l1_data_gas_price: header.eth_l1_data_gas_price,
            strk_l1_data_gas_price: header.strk_l1_data_gas_price,
            receipt_commitment: header.receipt_commitment,
            l1_da_mode: header.l1_da_mode,
            state_diff_commitment: header.state_diff_commitment,
        }
    }
}
fn to_64_bits(num: usize) -> [u8; 8] {
    let sized_transaction_count: u64 = num.try_into().expect("Expect usize is at most 8 bytes");
    sized_transaction_count.to_be_bytes()
}

pub fn compute_final_hash(header: &BlockHeaderData) -> Result<Felt, io::Error> {
    // Concatenate the transaction count, event count, state diff length, and L1
    // data availability mode into a single felt.

    let l1_data_availability_byte: u8 = match header.l1_da_mode {
        L1DataAvailabilityMode::Calldata => 0,
        L1DataAvailabilityMode::Blob => 0b10000000,
    };
    let concat_bytes = [
        to_64_bits(header.transaction_count as usize).as_slice(),
        to_64_bits(header.event_count as usize).as_slice(),
        to_64_bits(header.state_diff_length as usize).as_slice(),
        &[l1_data_availability_byte],
        &[0_u8; 7], // zero padding
    ]
    .concat();

    let concat_counts = Felt::from_bytes_be_slice(concat_bytes.as_slice());

    // Hash the block header.
    let data = vec![
        Felt::from_bytes_be_slice(b"STARKNET_BLOCK_HASH0"),
        header.number.into(),
        header.state_commitment,
        header.sequencer_address,
        header.timestamp.into(),
        concat_counts,
        header.state_diff_commitment,
        header.transaction_commitment,
        header.event_commitment,
        header.receipt_commitment,
        header.eth_l1_gas_price.into(),
        header.strk_l1_gas_price.into(),
        header.eth_l1_data_gas_price.into(),
        header.strk_l1_data_gas_price.into(),
        Felt::from_bytes_be_slice(header.starknet_version.as_bytes()),
        Felt::ZERO,
        header.parent_hash,
    ];
    let final_hash = Poseidon::hash_array(&data);

    Ok(final_hash)
}

/// Calculate transaction commitment hash value.
///
/// The transaction commitment is the root of the Patricia Merkle tree with
/// height 64 constructed by adding the (transaction_index,
/// transaction_hash_with_signature) key-value pairs to the tree and computing
/// the root hash.
pub fn calculate_transaction_commitment(transactions: &[TxnWithHash<Felt>]) -> Result<Felt> {
    use rayon::prelude::*;

    let final_hashes = transactions
        .par_iter()
        .map(calculate_transaction_hash_with_signature)
        .collect();

    calculate_commitment_root::<PoseidonHash>(final_hashes)
}

pub fn calculate_receipt_commitment(receipts: &[super::receipt::Receipt]) -> Result<Felt> {
    use rayon::prelude::*;

    let hashes = receipts
        .par_iter()
        .map(|receipt| {
            poseidon_hash_many(&[
                receipt.transaction_hash,
                receipt.actual_fee,
                // Calculate hash of messages sent.
                {
                    let mut hasher = PoseidonHasher::new();
                    hasher.update((receipt.l2_to_l1_messages.len() as u64).into());
                    for msg in &receipt.l2_to_l1_messages {
                        hasher.update(msg.from_address);
                        hasher.update(msg.to_address);
                        hasher.update((msg.payload.len() as u64).into());
                        for payload in &msg.payload {
                            hasher.update(*payload);
                        }
                    }
                    hasher.finalize()
                },
                // Revert reason.
                match &receipt.execution_status {
                    ExecutionStatus::Succeeded => Felt::ZERO,
                    ExecutionStatus::Reverted { reason } => {
                        let mut keccak = sha3::Keccak256::default();
                        keccak.update(reason.as_bytes());
                        let mut hashed_bytes: [u8; 32] = keccak.finalize().into();
                        hashed_bytes[0] &= 0b00000011_u8; // Discard the six MSBs.
                        Felt::from_bytes_be(&hashed_bytes)
                    }
                },
                // Execution resources:
                // L2 gas
                Felt::ZERO,
                // L1 gas consumed
                receipt.execution_resources.total_gas_consumed.l1_gas.into(),
                // L1 data gas consumed
                receipt
                    .execution_resources
                    .total_gas_consumed
                    .l1_data_gas
                    .into(),
            ])
            // .into()
        })
        .collect();

    calculate_commitment_root::<PoseidonHash>(hashes)
}

fn calculate_commitment_root<H: FeltHash>(hashes: Vec<Felt>) -> Result<Felt> {
    let mut tree: TransactionOrEventTree<H> = Default::default();

    hashes
        .into_iter()
        .enumerate()
        .try_for_each(|(idx, final_hash)| {
            let idx: u64 = idx
                .try_into()
                .expect("too many transactions while calculating commitment");
            tree.set(idx, final_hash)
        })
        .context("Building transaction commitment tree")?;

    tree.commit()
}

/// Compute the combined hash of the transaction hash and the signature.
///
/// [Reference code from StarkWare](https://github.com/starkware-libs/starknet-api/blob/5565e5282f5fead364a41e49c173940fd83dee00/src/block_hash/block_hash_calculator.rs#L95-L98).
fn calculate_transaction_hash_with_signature(tx: &TxnWithHash<Felt>) -> Felt {
    let signature = match &tx.transaction {
        // Matching for Invoke transactions with different versions
        Txn::Invoke(invoke_txn) => match invoke_txn {
            InvokeTxn::V0(tx) => tx.signature.as_slice(),
            InvokeTxn::V1(tx) => tx.signature.as_slice(),
            InvokeTxn::V3(tx) => tx.signature.as_slice(),
        },
        Txn::Declare(declare_txn) => match declare_txn {
            DeclareTxn::V0(tx) => tx.signature.as_slice(),
            DeclareTxn::V1(tx) => tx.signature.as_slice(),
            DeclareTxn::V2(tx) => tx.signature.as_slice(),
            DeclareTxn::V3(tx) => tx.signature.as_slice(),
        },
        Txn::DeployAccount(deploy_acc) => match deploy_acc {
            DeployAccountTxn::V1(tx) => tx.signature.as_slice(),
            DeployAccountTxn::V3(tx) => tx.signature.as_slice(),
        },
        Txn::Deploy(_) | Txn::L1Handler(_) => &[Felt::ZERO],
    };

    // Collect transaction hash and signature elements into a vector
    let mut data = vec![tx.transaction_hash];
    data.extend_from_slice(signature);

    // Compute the hash for the entire data vector
    Poseidon::hash_array(&data)
}

/// Calculate event commitment hash value.
///
/// The event commitment is the root of the Patricia Merkle tree with height 64
/// constructed by adding the (event_index, event_hash) key-value pairs to the
/// tree and computing the root hash.
pub fn calculate_event_commitment(
    transaction_events: &Vec<(TransactionHash, Vec<Event>)>,
) -> Result<Felt> {
    use rayon::prelude::*;

    let event_hashes = transaction_events
        .par_iter()
        .flat_map(|(tx_hash, events)| events.par_iter().map(|e| (*tx_hash, e)))
        .map(|(tx_hash, e)| calculate_event_hash(e, tx_hash))
        .collect();

    calculate_commitment_root::<PoseidonHash>(event_hashes)
}

/// Calculate the hash of an event.
/// [Reference code from StarkWare](https://github.com/starkware-libs/starknet-api/blob/5565e5282f5fead364a41e49c173940fd83dee00/src/block_hash/event_commitment.rs#L33).
fn calculate_event_hash(event: &Event, transaction_hash: Felt) -> Felt {
    let mut data = vec![
        event.from_address,
        transaction_hash,
        (event.keys.len() as u64).into(),
    ];

    // Add each key to the vector
    data.extend(event.keys.iter().copied());

    // Add the data length and each data element to the vector
    data.push((event.data.len() as u64).into());
    data.extend(event.data.iter().copied());

    // Compute the final hash
    Poseidon::hash_array(&data)
}
