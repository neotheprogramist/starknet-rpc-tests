pub mod args;
pub mod starknet;
pub mod utils;

use args::Args;
use clap::Parser;
use starknet::state::{starknet_config::StarknetConfig, Starknet};
use utils::{
    add_transaction_receipts, handle_transactions, read_transactions_file, write_result_state_file,
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let args = Args::parse();

    let mut starknet: Starknet = Starknet::new(&StarknetConfig::default(), &args.acc_path).unwrap();

    let transactions = read_transactions_file(&args.txns_path).unwrap();
    handle_transactions(&mut starknet, transactions);
    add_transaction_receipts(&mut starknet);
    write_result_state_file(&args.state_path, &starknet).unwrap();
}
