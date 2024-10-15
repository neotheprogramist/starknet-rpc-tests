use starknet_types_core::felt::FromStrError;
use thiserror::Error;

use crate::v7::rpc::{
    endpoints::{
        declare_contract::{ClassHashParseError, RunnerError},
        errors::{NonAsciiNameError, RpcError},
    },
    providers::provider::ProviderError,
};
#[derive(Error, Debug)]
pub enum DevnetError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error("The restart operation failed: {msg}")]
    Restart { msg: String },
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    NonAsciiName(#[from] NonAsciiNameError),
    #[error(transparent)]
    FromStr(#[from] FromStrError),
    #[error(transparent)]
    ClassHash(#[from] ClassHashParseError),
    #[error(transparent)]
    Regex(#[from] regex::Error),
}

impl From<RunnerError> for DevnetError {
    fn from(err: RunnerError) -> Self {
        DevnetError::Rpc(RpcError::RunnerError(err))
    }
}
