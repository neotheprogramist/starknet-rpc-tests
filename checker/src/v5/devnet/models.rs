use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_5_0::{BlockHash, FeeUnit, TxnHash};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountBalanceParams {
    pub address: Felt,
    pub unit: Option<FeeUnit>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountBalanceResponse {
    pub amount: Vec<u64>,
    pub unit: FeeUnit,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SerializableAccount {
    pub initial_balance: String,
    pub address: Felt,
    pub public_key: Felt,
    pub private_key: Felt,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SetTimeParams {
    pub time: u64,
    pub generate_block: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SetTimeResponse {
    pub block_timestamp: u64,
    pub block_hash: Option<BlockHash>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IncreaseTimeParams {
    pub time: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IncreaseTimeResponse {
    pub timestamp_increased_by: u64,
    pub block_hash: BlockHash,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MintTokensParams {
    pub address: Felt,
    pub amount: u128,
    pub unit: Option<FeeUnit>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MintTokensResponse {
    pub new_balance: String,
    pub unit: FeeUnit,
    pub tx_hash: TxnHash,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ForkStatusResponse {
    pub url: Option<String>,
    pub block: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateBlockResponse {
    pub block_hash: BlockHash,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbortBlocksParams {
    pub starting_block_hash: BlockHash,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbortBlocksResponse {
    pub aborted: Vec<BlockHash>,
}
