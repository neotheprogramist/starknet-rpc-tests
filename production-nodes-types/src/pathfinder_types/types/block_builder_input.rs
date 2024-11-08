use super::block::BlockHeader;
use crate::pathfinder_types::starknet::state_diff::StateDiff;
use serde::{Deserialize, Serialize};
use starknet_devnet_types::rpc::transaction_receipt::TransactionReceipt;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::starknet_api_openrpc::TxnWithHash;

pub type TransactionHash = Felt;
pub type BlockHash = Felt;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StarknetBlock {
    pub header: BlockHeader,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ThinStarknetBlocks {
    pub header: BlockHeader,
    pub state_diff: StateDiff,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct B11rInput {
    pub blocks: ThinStarknetBlocks,
    pub transactions: Vec<TxnWithHash<Felt>>,
    pub transaction_receipts: Vec<TransactionReceipt>,
}
