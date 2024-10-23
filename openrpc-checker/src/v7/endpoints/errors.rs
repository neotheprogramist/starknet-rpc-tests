use starknet_types_core::felt::FromStrError;
use thiserror::Error;

use super::declare_contract::RunnerError;
use crate::v7::{
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
    AccountError_(#[from] AccountError<crate::v7::accounts::single_owner::SignError<SignError>>),
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
    #[error("Unexpected txn type {0}")]
    UnexpectedTxnType(String),
    #[error("TxnExecutionStatus reverted {0}")]
    TxnExecutionStatus(String),
    #[error("Required input not provided {0}")]
    InvalidInput(String),
    #[error("Timeout waiting for tx receipt {0}")]
    Timeout(String),
    #[error("Txn rejected {0}")]
    TransactionRejected(String),
    #[error("Txn failed {0}")]
    TransactionFailed(String),
    #[error("Empty url error {0}")]
    EmptyUrlList(String),
    #[error("Unexpected error occured: {0}")]
    Other(String),
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CallError {
    #[error("Error creating an account")]
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
