pub mod block_build;
use block_build::block::build_block_tx_hashes;
use serde_json;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::*;

fn main() {
    let block = build_block_tx_hashes(
        "b11r/testdata/header.json",
        "b11r/testdata/txs.json",
        BlockStatus::Pending,
    )
    .unwrap();

    let pretty_json = serde_json::to_string_pretty(&block).unwrap();

    println!("{}", pretty_json);
}
