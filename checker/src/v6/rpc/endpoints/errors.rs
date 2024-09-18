use starknet_types_core::felt::FromStrError;
use thiserror::Error;

use super::declare_contract::RunnerError;
use crate::v6::rpc::{
    accounts::{account::AccountError, errors::CreationError, utils::mint::MintError},
    providers::provider::ProviderError,
    signers::local_wallet::SignError,
};
use core::fmt::{Display, Formatter, Result};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum RpcError {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    RunnerError(#[from] RunnerError),
    #[error(transparent)]
    CreationError(#[from] CreationError),
    #[error(transparent)]
    MintError(#[from] MintError),
    #[error(transparent)]
    SignError(#[from] SignError),
    #[error(transparent)]
    AccountError(#[from] AccountError<SignError>),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    CallError(#[from] CallError),
    #[error(transparent)]
    NonAsciiNameError(#[from] NonAsciiNameError),
    #[error(transparent)]
    FromStrError(#[from] FromStrError),
    #[error("Unexpected block type {0}")]
    UnexpectedBlockResponseType(String),
    #[error("TxnExecutionStatus reverted {0}")]
    TxnExecutionStatus(String),
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CallError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error(transparent)]
    ProviderError(#[from] ProviderError),

    #[error(transparent)]
    FromStrError(#[from] FromStrError),

    #[error(transparent)]
    RunnerError(#[from] RunnerError),

    #[error("Unexpected receipt response type")]
    UnexpectedReceiptType,

    #[error("Unexpected execution result")]
    UnexpectedExecutionResult,
}

#[derive(Debug)]
pub struct NonAsciiNameError;

impl std::error::Error for NonAsciiNameError {}

impl Display for NonAsciiNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "the provided name contains non-ASCII characters")
    }
}
