use starknet_types_core::felt::FromStrError;
use thiserror::Error;

use super::declare_contract::RunnerError;
use crate::v5::rpc::{
    accounts::{account::AccountError, errors::CreationError, utils::mint::MintError},
    providers::provider::ProviderError,
    signers::local_wallet::SignError,
};
use core::fmt::{Display, Formatter, Result};

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Runner error {0}")]
    RunnerError(#[from] RunnerError),
    #[error("Creation error {0}")]
    CreationError(#[from] CreationError),
    #[error("Minting error {0}")]
    MintError(#[from] MintError),
    #[error("Sign error {0}")]
    SignError(#[from] SignError),
    #[error("Account error {0}")]
    AccountError(#[from] AccountError<SignError>),
    #[error("Provider error {0}")]
    ProviderError(#[from] ProviderError),
    #[error("Call error {0}")]
    CallError(#[from] CallError),
    #[error("Non Ascii Name error {0}")]
    NonAsciiNameError(#[from] NonAsciiNameError),
    #[error("From Str error {0}")]
    FromStrError(#[from] FromStrError),
    #[error("Unexpected block type {0}")]
    UnexpectedBlockResponseType(String),
    #[error("TxnExecutionStatus reverted {0}")]
    TxnExecutionStatus(String),
}

#[derive(Error, Debug)]
pub enum CallError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
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
