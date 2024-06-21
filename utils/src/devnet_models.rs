use std::{fmt::Display, num::NonZeroU128};

use num_bigint::BigUint;
use serde::{Deserialize, Serialize, Serializer};
use starknet_crypto::FieldElement;
use url::Url;

use crate::{models::ContractClass, starknet_utils::parse_cairo_short_string};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DevnetMethod {
    #[serde(rename = "get_account_balance")]
    GetAccountBalance,
    #[serde(rename = "config")]
    Config,
}
#[derive(Debug, Serialize)]
struct MintRequest {
    amount: u128,
    address: String,
}
#[derive(Debug, Serialize)]
struct IncreaseTimeRequest {
    time: u64,
}
#[derive(Debug, Serialize)]
struct FlushRequest {
    dry_run: bool,
}
#[derive(Debug, Serialize)]
struct SetTimeRequest {
    time: u64,
    generate_block: bool,
}
#[derive(Debug, Serialize)]
struct AbortBlocksRequest {
    starting_block_hash: String,
}
#[derive(Debug, Serialize)]
struct LoadRequest {
    network_url: String,
    address: Option<String>,
}
#[derive(Debug, Serialize)]
struct ConsumeMessageRequest {
    l2_contract_address: String,
    l1_contract_address: String,
    payload: Vec<String>,
}
#[derive(Debug, Serialize)]
struct SendMessageRequest {
    l2_contract_address: String,
    entry_point_selector: String,
    l1_contract_address: String,
    payload: Vec<String>,
    paid_fee_on_l1: String,
    nonce: String,
}
pub type Balance = BigUint;
pub const MAINNET: FieldElement = FieldElement::from_mont([
    17696389056366564951,
    18446744073709551615,
    18446744073709551615,
    502562008147966918,
]);

pub const TESTNET: FieldElement = FieldElement::from_mont([
    3753493103916128178,
    18446744073709548950,
    18446744073709551615,
    398700013197595345,
]);

pub const TESTNET2: FieldElement = FieldElement::from_mont([
    1663542769632127759,
    18446744073708869172,
    18446744073709551615,
    33650220878420990,
]);

pub const SEPOLIA: FieldElement = FieldElement::from_mont([
    1555806712078248243,
    18446744073708869172,
    18446744073709551615,
    507980251676163170,
]);
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
#[clap(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChainId {
    Mainnet,
    Testnet,
}

impl ChainId {
    pub fn goerli_legacy_id() -> FieldElement {
        TESTNET.into()
    }

    pub fn to_felt(&self) -> FieldElement {
        FieldElement::from(self).into()
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let felt = FieldElement::from(self);
        let str = parse_cairo_short_string(&felt).map_err(|_| std::fmt::Error)?;
        f.write_str(&str)
    }
}

impl From<ChainId> for FieldElement {
    fn from(value: ChainId) -> Self {
        match value {
            ChainId::Mainnet => MAINNET,
            ChainId::Testnet => SEPOLIA,
        }
    }
}

impl From<&ChainId> for FieldElement {
    fn from(value: &ChainId) -> Self {
        match value {
            ChainId::Mainnet => MAINNET,
            ChainId::Testnet => SEPOLIA,
        }
    }
}

pub fn serialize_initial_balance<S>(balance: &Balance, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&balance.to_str_radix(10))
}

pub fn serialize_chain_id<S>(chain_id: &ChainId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{chain_id}"))
}
#[derive(Clone, Debug, Serialize)]
pub struct StarknetConfig {
    pub seed: u32,
    pub total_accounts: u8,
    #[serde(skip_serializing)]
    pub account_contract_class: ContractClass,
    pub account_contract_class_hash: FieldElement,
    #[serde(serialize_with = "serialize_initial_balance")]
    pub predeployed_accounts_initial_balance: Balance,
    pub start_time: Option<u64>,
    pub gas_price_wei: NonZeroU128,
    pub gas_price_strk: NonZeroU128,
    pub data_gas_price_wei: NonZeroU128,
    pub data_gas_price_strk: NonZeroU128,
    #[serde(serialize_with = "serialize_chain_id")]
    pub chain_id: ChainId,
    pub dump_on: Option<DumpOn>,
    pub dump_path: Option<String>,
    pub blocks_on_demand: bool,
    pub lite_mode: bool,
    /// on initialization, re-execute loaded txs (if any)
    #[serde(skip_serializing)]
    pub re_execute_on_init: bool,
    pub state_archive: StateArchiveCapacity,
    pub fork_config: ForkConfig,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DumpOn {
    Exit,
    Block,
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum StateArchiveCapacity {
    #[default]
    None,
    Full,
}

pub fn serialize_config_url<S>(url: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match url {
        Some(url) => serializer.serialize_str(url.as_ref()),
        None => serializer.serialize_none(),
    }
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct ForkConfig {
    #[serde(serialize_with = "serialize_config_url")]
    pub url: Option<Url>,
    pub block_number: Option<u64>,
}
