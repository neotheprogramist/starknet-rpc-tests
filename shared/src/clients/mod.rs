use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

pub mod devnet_client;

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
