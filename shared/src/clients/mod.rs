use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

pub mod devnet_client;
pub mod devnet_provider;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    seed: u64,
    total_accounts: u32,
    account_contract_class_hash: String,
    predeployed_accounts_initial_balance: String,
    start_time: Option<String>,
    gas_price_wei: u64,
    gas_price_strk: u64,
    data_gas_price_wei: u64,
    data_gas_price_strk: u64,
    chain_id: String,
    dump_on: String,
    dump_path: String,
    state_archive: String,
    fork_config: ForkConfig,
    server_config: ServerConfig,
    blocks_on_demand: bool,
    lite_mode: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForkConfig {
    url: String,
    block_number: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    host: String,
    port: u16,
    timeout: u64,
    request_body_size_limit: u64,
}

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum DevnetClientError<T> {
    #[error(transparent)]
    JsonError(serde_json::Error),
    #[error(transparent)]
    TransportError(T),
    #[error(transparent)]
    DevnetError(DevnetError),
}
#[derive(Debug, Deserialize)]
pub struct DevnetError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
impl Error for DevnetError {}
impl Display for DevnetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            Some(data) => {
                write!(
                    f,
                    "JSON-RPC error: code={}, message=\"{}\", data={}",
                    self.code,
                    self.message,
                    serde_json::to_string(data).map_err(|_| std::fmt::Error)?
                )
            }
            None => {
                write!(
                    f,
                    "JSON-RPC error: code={}, message=\"{}\"",
                    self.code, self.message
                )
            }
        }
    }
}
