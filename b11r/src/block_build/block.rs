use serde_json::Result;
use starknet_types_core::hash::{Pedersen, StarkHash};
use std::fs::File;
use std::io::Read;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;
use starknet_types_core::felt::Felt;


pub fn build_block_tx_hashes(block_header_path: &str, transactions_path: &str, status: BlockStatus) -> Result<BlockWithTxHashes<Felt>>{

    let mut header_file = File::open(block_header_path).expect("Unable to open file");
    
    let mut contents = String::new();
    header_file.read_to_string(&mut contents).expect("Unable to read file");
    let block_header: BlockHeader<Felt> = serde_json::from_str(&contents)?;

    let mut transactions_file = File::open(transactions_path).expect("Unable to open file");
    
    contents = String::new();
    transactions_file.read_to_string(&mut contents).expect("Unable to read file");
    let transactions: Vec<TxnHash<Felt>> = serde_json::from_str(&contents)?;


    let mut block = BlockWithTxHashes {
        block_header,
        status,
        transactions,
    };

    block.block_header.block_hash = generate_hash(&block)?;
    
    Ok(block)
}

fn generate_hash(block: &BlockWithTxHashes<Felt>) -> Result<Felt> {
    let hash = Pedersen::hash_array(&[
        Felt::from(block.block_header.block_number),           // block number
        block.block_header.new_root,                    // global_state_root
        block.block_header.sequencer_address,              // sequencer_address
        Felt::from_dec_str(&block.block_header.timestamp.to_string()).unwrap(),              // block_timestamp
        Felt::from(block.transactions.len() as u64), // transaction_count
        Felt::ZERO,                                  // transaction_commitment
        Felt::ZERO,                                  // event_count
        Felt::ZERO,                                  // event_commitment
        Felt::ZERO,                                  // protocol_version
        Felt::ZERO,                                  // extra_data
        block.block_header.parent_hash,                   // parent_block_hash
    ]);
    Ok(hash)
}

