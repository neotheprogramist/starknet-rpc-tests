mod args;
pub mod block_build;
use args::Args;
use block_build::block::build_block_tx_hashes;
use clap::Parser;
use serde_json;

fn main() {
    //target/b11r
    let args = Args::parse();

    let block = build_block_tx_hashes(
        args.block_header_path,
        args.transactions_path,
        args.receipt_path,
        args.state_diff_path,
    )
    .unwrap();

    let pretty_json = serde_json::to_string_pretty(&block).unwrap();

    println!("{}", pretty_json);
}
