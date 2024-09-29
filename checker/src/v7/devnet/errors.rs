use starknet_types_core::felt::FromStrError;
use thiserror::Error;

use crate::v7::rpc::{
    endpoints::errors::{NonAsciiNameError, RpcError},
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
}
