use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionsError {
    #[error("Conversion failed: {0}")]
    FeltVecToBigUintError(String),
}
