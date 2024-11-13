use production_nodes_types::pathfinder_types::{
    starknet::state_diff::StateDiff,
    types::{
        block::{Block, BlockHeader, BlockHeaderData},
        block_builder_input::{B11rInput, BlockHash},
        block_hash::{
            calculate_event_commitment, calculate_receipt_commitment,
            calculate_transaction_commitment, compute_final_hash,
        },
        event::{extract_emmited_events, get_events_count, Event},
        receipt::{convert_receipts, Receipt},
        reply::{state_update::StateDiff as GatewayStateDiff, StateUpdate as GatewayStateUpdate},
        state_update::{state_diff_commitment::compute, StateUpdate},
    },
};

use starknet_devnet_types::rpc::transaction_receipt::TransactionReceipt;

use starknet_types_core::felt::Felt;

use starknet_types_rpc::v0_7_1::starknet_api_openrpc::TxnWithHash;

use super::errors::Error;

pub fn build_block_tx_hashes_thin(b11r_input: B11rInput) -> Result<Block, Error> {
    let block_header: BlockHeader = b11r_input.blocks.header;

    let transactions: Vec<TxnWithHash<Felt>> = b11r_input.transactions;

    let transaction_receipts: Vec<TransactionReceipt> = b11r_input.transaction_receipts;

    let events = extract_emmited_events(transaction_receipts.clone());

    let transaction_hash_with_events: Vec<(Felt, Vec<Event>)> = events
        .clone()
        .into_iter()
        .map(|emitted_event| (emitted_event.transaction_hash, emitted_event.events))
        .collect();

    let (block_hash, block_state_diff): (BlockHash, StateDiff) =
        (block_header.block_hash, b11r_input.blocks.state_diff);

    let state_diff_gateway: GatewayStateDiff = block_state_diff.into();

    let state_update_gateway = GatewayStateUpdate::new(block_hash, state_diff_gateway);
    let state_update_common: StateUpdate = state_update_gateway.into();

    let state_diff_commitment = compute(
        &state_update_common.contract_updates,
        &state_update_common.system_contract_updates,
        &state_update_common.declared_cairo_classes,
        &state_update_common.declared_sierra_classes,
    );

    let receipts: Vec<Receipt> = convert_receipts(transaction_receipts.clone())
        .into_iter()
        .map(|receipt| receipt.into())
        .collect();

    let receipt_commitment = calculate_receipt_commitment(&receipts)?;

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
            block_header
                .l1_gas_price
                .price_in_wei
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        strk_l1_gas_price: u128::from_str_radix(
            block_header
                .l1_gas_price
                .price_in_fri
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        eth_l1_data_gas_price: u128::from_str_radix(
            block_header
                .l1_data_gas_price
                .price_in_wei
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        strk_l1_data_gas_price: u128::from_str_radix(
            block_header
                .l1_data_gas_price
                .price_in_fri
                .as_str()
                .trim_start_matches("0x"),
            16,
        )?,
        receipt_commitment,
        l1_da_mode: block_header.l1_da_mode,
    };

    block_header_data.hash = compute_final_hash(&block_header_data)?;

    let block = Block {
        header: block_header_data,
        transactions,
    };
    Ok(block)
}
