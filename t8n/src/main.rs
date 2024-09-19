pub mod args;
pub mod starknet;
pub mod utils;
use crate::starknet::state::errors::Error;
use args::Args;
use clap::Parser;
use starknet::state::{starknet_config::StarknetConfig, Starknet};
use utils::{
    add_transaction_receipts, handle_transactions, read_transactions_file, write_result_state_file,
};

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let args = Args::parse();

    let mut starknet: Starknet = Starknet::new(&StarknetConfig::default(), &args.acc_path)?;

    let transactions = read_transactions_file(&args.txns_path)?;
    handle_transactions(&mut starknet, transactions)?;
    add_transaction_receipts(&mut starknet)?;
    write_result_state_file(&args.state_path, &starknet)?;

    Ok(())
}
