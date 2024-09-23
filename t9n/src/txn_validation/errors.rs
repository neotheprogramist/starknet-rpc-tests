use crypto_utils::curve::signer::VerifyError;
use serde_json;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("Resource name is not a string")]
    ResourceNameError,
    #[error(transparent)]
    VerifyError(#[from] VerifyError),
}
