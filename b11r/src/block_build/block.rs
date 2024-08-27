use serde_json::Result;
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


    let block = BlockWithTxHashes {
        block_header,
        status,
        transactions,
    };
    
    Ok(block)
}

