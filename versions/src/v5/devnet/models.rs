use serde::{Deserialize, Serialize};
use starknet_types_rpc::{FeeUnit, Felt};

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
