pub mod args;
pub mod starknet;
pub mod utils;

use crate::starknet::state::errors::Error;
use args::Args;
use clap::Parser;
use starknet::state::{
    starknet_config::StarknetConfig, starknet_state::StateWithBlockNumber, Starknet,
};
use utils::{
    add_transaction_receipts, handle_transactions, read_state_file, read_transactions_file,
    write_result_state_file,
};

fn initialize_starknet(args: &Args) -> Result<Starknet, Error> {
    if args.forwarded_state {
        let state_with_block_number: StateWithBlockNumber = read_state_file(&args.state_path)?;
        Starknet::from_init_state(state_with_block_number)
    } else {
        Starknet::new(
            &StarknetConfig::default(),
            args.acc_path.as_ref().ok_or(Error::AccPathNotProvided)?,
        )
    }
}

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    let mut starknet = initialize_starknet(&args)?;

    let transactions = read_transactions_file(&args.txns_path)?;

    handle_transactions(&mut starknet, transactions)?;
    add_transaction_receipts(&mut starknet)?;
    write_result_state_file(&args.state_path, &starknet)?;

    Ok(())
}
