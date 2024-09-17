use pathfinder_types::types::block_hash::{
    calculate_event_commitment, calculate_receipt_commitment, calculate_transaction_commitment,
    compute_final_hash,
};

use pathfinder_types::starknet::state_diff::BlockStateDiff;
use pathfinder_types::types::block::{Block, BlockHeader, BlockHeaderData};
use pathfinder_types::types::event::{extract_emmited_events, get_events_count, Event};
use pathfinder_types::types::receipt::convert_receipts;
use pathfinder_types::types::reply::state_update::StateDiff as GatewayStateDiff;
use pathfinder_types::types::reply::StateUpdate as GatewayStateUpdate;
use pathfinder_types::types::state_update::{state_diff_commitment::compute, StateUpdate};
use starknet_devnet_types::rpc::transaction_receipt::TransactionReceipt;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::TxnWithHash;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn build_block_tx_hashes(
    block_header_path: PathBuf,
    transactions_path: PathBuf,
    receipt_path: PathBuf,
    state_diff_path: PathBuf,
) -> Result<Block, Box<dyn std::error::Error>> {
    let mut header_file = File::open(block_header_path).expect("Unable to open file");

    let mut header = String::new();
    header_file
        .read_to_string(&mut header)
        .expect("Unable to read file");

    let mut transactions_file = File::open(transactions_path).expect("Unable to open file");

    let mut txns = String::new();
    transactions_file
        .read_to_string(&mut txns)
        .expect("Unable to read file");

    let mut receipts_file = File::open(receipt_path).expect("Unable to open file");

    let mut receipts = String::new();
    receipts_file
        .read_to_string(&mut receipts)
        .expect("Unable to read file");

    let mut state_diff_file = File::open(state_diff_path).expect("Unable to open file");

    let mut state_diff = String::new();
    state_diff_file
        .read_to_string(&mut state_diff)
        .expect("Unable to read file");

    let block_header: BlockHeader = serde_json::from_str(&header)?;

    let transactions: Vec<TxnWithHash<Felt>> = serde_json::from_str(&txns)?;

    let transaction_receipts: Vec<TransactionReceipt> = serde_json::from_str(&receipts)?;

    let events = extract_emmited_events(transaction_receipts.clone());

    let transaction_hash_with_events: Vec<(Felt, Vec<Event>)> = events
        .clone()
        .into_iter()
        .map(|emitted_event| (emitted_event.transaction_hash, emitted_event.events))
        .collect();

    let block_state_diff: BlockStateDiff = serde_json::from_str(&state_diff)?;
    let block_hash = block_state_diff.hash_to_state_diff.block_hash;
    let state_diff = block_state_diff.hash_to_state_diff.state_diff;
    let state_diff_gateway: GatewayStateDiff = state_diff.into();

    let state_update_gateway = GatewayStateUpdate::new(block_hash, state_diff_gateway);

    let state_update_common: StateUpdate = state_update_gateway.into();

    let state_diff_commitment = compute(
        &state_update_common.contract_updates,
        &state_update_common.system_contract_updates,
        &state_update_common.declared_cairo_classes,
        &state_update_common.declared_sierra_classes,
    );

    let mut block_header_data: BlockHeaderData = BlockHeaderData {
        hash: Default::default(),
        parent_hash: block_header.parent_hash,
        number: block_header.block_number,
        timestamp: block_header.timestamp,
        sequencer_address: block_header.sequencer,
        state_commitment: block_header.state_root,
        state_diff_commitment,
        transaction_commitment: calculate_transaction_commitment(&transactions)?,
        transaction_count: transactions.len() as u32,
        event_commitment: calculate_event_commitment(&transaction_hash_with_events)?,
        event_count: get_events_count(events.clone()),
        state_diff_length: state_update_common.state_diff_length(),
        starknet_version: block_header.starknet_version,
        eth_l1_gas_price: u128::from_str_radix(
            &block_header
                .l1_gas_price
                .price_in_wei
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        strk_l1_gas_price: u128::from_str_radix(
            &&block_header
                .l1_gas_price
                .price_in_fri
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        eth_l1_data_gas_price: u128::from_str_radix(
            &block_header
                .l1_data_gas_price
                .price_in_wei
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        strk_l1_data_gas_price: u128::from_str_radix(
            &&block_header
                .l1_data_gas_price
                .price_in_fri
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        receipt_commitment: calculate_receipt_commitment(&convert_receipts(
            transaction_receipts.clone(),
        ))?,
        l1_da_mode: block_header.l1_da_mode,
    };

    block_header_data.hash = compute_final_hash(&block_header_data)?;

    let block = Block {
        header: block_header_data,
        transactions,
    };
    Ok(block)
}
