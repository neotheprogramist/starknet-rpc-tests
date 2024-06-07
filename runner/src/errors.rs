use thiserror::Error;
use url::ParseError;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("failed to parse url")]
    ParsingError(#[from] ParseError),

    #[error("SerdeJsonError error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("ReadFileError error: {0}")]
    ReadFileError(String),

    #[error("Account error: {0}")]
    AccountFailure(String),

    #[error("Deployment error: {0}")]
    DeploymentFailure(String),

    #[error("Box error: {0}")]
    BoxError(#[from] Box<dyn std::error::Error>),

    #[error("Starknet-devnet not launched : {0}")]
    DevnetNotLaunched(String),

    #[error("Request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
