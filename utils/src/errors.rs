use regex::Regex;
use starknet_crypto::FieldElement;
use thiserror::Error;
use url::ParseError;

#[derive(Debug, Error)]
#[allow(dead_code)]
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
pub fn parse_class_hash_from_error(error_msg: &str) -> FieldElement {
    tracing::info!("Error message: {}", error_msg);
    let re = Regex::new(r#"StarkFelt\("(0x[a-fA-F0-9]+)"\)"#).unwrap();

    // Attempt to capture the class hash
    if let Some(captures) = re.captures(error_msg) {
        if let Some(contract_address) = captures.get(1) {
            return FieldElement::from_hex_be(contract_address.as_str())
                .expect("Failed to parse class hash");
        }
    }

    panic!("Failed to extract class hash from error message");
}
