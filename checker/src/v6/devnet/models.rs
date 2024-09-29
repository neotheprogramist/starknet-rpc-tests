use num_bigint::BigUint;
use serde::{Deserialize, Deserializer, Serialize};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_6_0::{BlockHash, MsgToL1, PriceUnit, TxnHash};
use std::net::IpAddr;
use std::num::NonZeroU128;
use std::str::FromStr;
use url::Url;

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
    pub unit: Option<PriceUnit>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MintTokensResponse {
    pub new_balance: String,
    pub unit: PriceUnit,
    pub tx_hash: TxnHash,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct DevnetConfigResponse {
    #[serde(flatten)]
    pub starknet_config: StarknetConfig,
    pub server_config: ServerConfig,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct StarknetConfig {
    pub seed: u32,
    pub total_accounts: u8,
    pub account_contract_class_hash: Felt,
    #[serde(deserialize_with = "deserialize_biguint")]
    pub predeployed_accounts_initial_balance: BigUint,
    pub start_time: Option<u64>,
    #[serde(deserialize_with = "deserialize_non_zero_u128")]
    pub gas_price_wei: NonZeroU128,
    #[serde(deserialize_with = "deserialize_non_zero_u128")]
    pub gas_price_strk: NonZeroU128,
    #[serde(deserialize_with = "deserialize_non_zero_u128")]
    pub data_gas_price_wei: NonZeroU128,
    #[serde(deserialize_with = "deserialize_non_zero_u128")]
    pub data_gas_price_strk: NonZeroU128,
    pub chain_id: String,
    pub dump_on: Option<DumpOn>,
    pub dump_path: Option<String>,
    pub blocks_on_demand: bool,
    pub lite_mode: bool,
    pub state_archive: StateArchiveCapacity,
    pub fork_config: ForkConfig,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DumpOn {
    Exit,
    Block,
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Deserialize)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum StateArchiveCapacity {
    #[default]
    None,
    Full,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ForkConfig {
    #[serde(deserialize_with = "deserialize_config_url")]
    pub url: Option<Url>,
    pub block_number: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub timeout: u16,
    pub request_body_size_limit: usize,
}

pub fn deserialize_config_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| Url::parse(&s).map_err(serde::de::Error::custom))
        .transpose()
}

fn deserialize_non_zero_u128<'de, D>(deserializer: D) -> Result<NonZeroU128, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u64 = Deserialize::deserialize(deserializer)?;
    NonZeroU128::new(value as u128)
        .ok_or_else(|| serde::de::Error::custom("Expected non-zero value"))
}

fn deserialize_biguint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    BigUint::from_str(&s).map_err(serde::de::Error::custom)
}
