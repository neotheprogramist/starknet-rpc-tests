use serde::Serialize;
use starknet_devnet_types::rpc::state::ThinStateDiff;

use crate::starknet::state::add_declare_transaction::add_declare_transaction;
use crate::starknet::state::add_deploy_account_transaction::add_deploy_account_transaction;
use crate::starknet::state::add_invoke_transaction::add_invoke_transaction;
use crate::starknet::state::{state_diff, Starknet};
use starknet_devnet_types::rpc::transaction_receipt::TransactionReceipt;
use starknet_devnet_types::rpc::transactions::BroadcastedTransaction;
use starknet_rs_core::types::BlockId;
use std::error::Error;
use std::fs;
use std::{fs::File, io::BufReader};
use tracing::{error, info};

pub fn read_transactions_file(
    file_path: &str,
) -> Result<Vec<BroadcastedTransaction>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let transactions: Vec<BroadcastedTransaction> = serde_json::from_reader(reader)?;
    Ok(transactions)
}

pub fn add_transaction_receipts(starknet: &mut Starknet) {
    let mut receipts: Vec<TransactionReceipt> = vec![];
    for starknet_transaction in starknet.transactions.iter() {
        let (_, transaction) = starknet_transaction;
        receipts.push(transaction.get_receipt().unwrap());
    }
    starknet.transaction_receipts = receipts;
}

pub fn handle_transactions(starknet: &mut Starknet, transactions: Vec<BroadcastedTransaction>) {
    // let mut starknet_state = starknet.state.clone();
    for (index, transaction) in transactions.into_iter().enumerate() {
        match transaction {
            BroadcastedTransaction::Invoke(tx) => match add_invoke_transaction(starknet, tx) {
                Err(e) => {
                    tracing::error!(
                        "Error processing Invoke transaction at index {}: {:?}",
                        index,
                        e
                    );
                }
                Ok(_) => {
                    tracing::info!(
                        "Successfully processed Invoke transaction at index {}",
                        index
                    );
                }
            },
            BroadcastedTransaction::Declare(tx) => match add_declare_transaction(starknet, tx) {
                Err(e) => {
                    tracing::error!(
                        "Error processing Declare transaction at index {}: {:?}",
                        index,
                        e
                    );
                }
                Ok(_) => {
                    tracing::info!(
                        "Successfully processed Declare transaction at index {}",
                        index
                    );
                }
            },
            BroadcastedTransaction::DeployAccount(tx) => {
                match add_deploy_account_transaction(starknet, tx) {
                    Err(e) => {
                        tracing::error!(
                            "Error processing DeployAccount transaction at index {}: {:?}",
                            index,
                            e
                        );
                    }
                    Ok(_) => {
                        tracing::info!(
                            "Successfully processed DeployAccount transaction at index {}",
                            index
                        );
                    }
                }
            }
        }
    }
    let state_diff = starknet.state.commit_with_diff().unwrap();
    let _ = starknet.generate_new_block(state_diff.clone());
    let state_thin: ThinStateDiff = state_diff.into();
    println!("State diff: {:?}", state_thin);
    // let _ = starknet.generate_new_block(starknet.block_state_update(&BlockId::Number(starknet.pending_block().header.block_number.0)).unwrap().state_diff);

    // starknet_state.rpc_contract_classes = starknet.state.rpc_contract_classes.clone();
    // starknet_state.state = starknet.state.state.clone();
    // println!("rpc_contract_classes {:?}", starknet_state.rpc_contract_classes);

    // let mut rpc_contract_classes = starknet.state.rpc_contract_classes.clone();
    // let state_diff = starknet_state.commit_with_diff().unwrap();
    // println!("State diff: {:?}", state_diff);
    // starknet.generate_new_block(state_diff).unwrap();
}

pub fn write_result_state_file<T: Serialize>(
    file_path: &str,
    data: &T,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(file_path)?;
    serde_json::to_writer_pretty(&file, data).map_err(|e| {
        error!("Failed to write JSON to file: {}", e);
        e
    })?;

    info!("State written into {}", file_path);
    Ok(())
}
