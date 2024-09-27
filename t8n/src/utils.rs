use crate::starknet::state::add_declare_transaction::add_declare_transaction;
use crate::starknet::state::add_deploy_account_transaction::add_deploy_account_transaction;
use crate::starknet::state::add_invoke_transaction::add_invoke_transaction;
use crate::starknet::state::errors::Error;
use crate::starknet::state::starknet_state::{StateWithBlock, StateWithBlockNumber};
use crate::starknet::state::Starknet;
use serde::Serialize;
use starknet_devnet_types::rpc::transaction_receipt::TransactionReceipt;
use starknet_devnet_types::rpc::transactions::BroadcastedTransaction;
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io::BufReader,
};
use tracing::{error, info};

pub fn read_state_file(file_path: &PathBuf) -> Result<StateWithBlockNumber, Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let state_with_block: StateWithBlock = serde_json::from_reader(reader)?;
    let state_with_block_number = StateWithBlockNumber {
        state: state_with_block.state,
        block_number: state_with_block.blocks.header.block_number,
    };
    Ok(state_with_block_number)
}

pub fn read_transactions_file(file_path: &PathBuf) -> Result<Vec<BroadcastedTransaction>, Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let transactions: Vec<BroadcastedTransaction> = serde_json::from_reader(reader)?;
    Ok(transactions)
}

pub fn add_transaction_receipts(starknet: &mut Starknet) -> Result<(), Error> {
    let mut receipts: Vec<TransactionReceipt> = vec![];
    for starknet_transaction in starknet.transactions.iter() {
        let (_, transaction) = starknet_transaction;
        receipts.push(transaction.get_receipt()?);
    }
    starknet.transaction_receipts = receipts;
    Ok(())
}

pub fn handle_transactions(
    starknet: &mut Starknet,
    transactions: Vec<BroadcastedTransaction>,
) -> Result<(), Error> {
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
    let state_diff = starknet.state.commit_with_diff()?;
    starknet.generate_new_block(state_diff.clone())?;
    Ok(())
}

pub fn write_result_state_file<T: Serialize>(file_path: &PathBuf, data: &T) -> Result<(), Error> {
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(file_path)?;
    serde_json::to_writer_pretty(&file, data).map_err(|e| {
        error!("Failed to write JSON to file: {}", e);
        e
    })?;

    info!("State written into {:?}", file_path);
    Ok(())
}
