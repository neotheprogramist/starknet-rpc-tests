use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{
    v0_5_0::{MsgToL1, TxnHash},
    v0_6_0::PriceUnit,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountBalanceParams {
    pub address: Felt,
    pub unit: Option<PriceUnit>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountBalanceResponse {
    pub amount: String,
    pub unit: PriceUnit,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SerializableAccount {
    pub initial_balance: String,
    pub address: Felt,
    pub public_key: Felt,
    pub private_key: Felt,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DumpPath {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoadPath {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostmanLoadL1MessagingContractParams {
    pub network_url: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostmanLoadL1MessagingContractResponse {
    pub messaging_contract_address: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostmanFlushParameters {
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanFlushResponse {
    pub messages_to_l1: Vec<MsgToL1>,
    pub messages_to_l2: Vec<MsgToL2>,
    pub generated_l2_transactions: Vec<TxnHash>,
    pub l1_provider: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct MsgToL2 {
    pub l2_contract_address: Felt,
    pub entry_point_selector: Felt,
    pub l1_contract_address: Felt,
    pub payload: Vec<Felt>,
    pub paid_fee_on_l1: Felt,
    pub nonce: Felt,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostmanSendMessageToL2Response {
    pub transaction_hash: TxnHash,
}
