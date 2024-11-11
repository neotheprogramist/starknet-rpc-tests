use super::{block::ClassHash, block_builder_input::TransactionHash};

/// API response for an INVOKE_FUNCTION transaction
#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvokeResponse {
    pub code: String, // TRANSACTION_RECEIVED
    pub transaction_hash: TransactionHash,
}

/// API response for a DECLARE transaction
#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeclareResponse {
    pub code: String, // TRANSACTION_RECEIVED
    pub transaction_hash: TransactionHash,
    pub class_hash: ClassHash,
}

/// API response for a DEPLOY ACCOUNT transaction
#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
pub struct DeployAccountResponse {
    pub code: String, // TRANSACTION_RECEIVED
    pub transaction_hash: TransactionHash,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AddTransactionResponseType {
    #[serde(rename = "INVOKE_FUNCTION")]
    Invoke,
    #[serde(rename = "DECLARE")]
    Declare,
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount,
}
