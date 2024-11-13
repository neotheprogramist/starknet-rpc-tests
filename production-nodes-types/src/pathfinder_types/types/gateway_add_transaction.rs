use super::{block::ClassHash, block_builder_input::TransactionHash};
use std::fmt;

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
pub enum AddTransactionRequestType {
    #[serde(rename = "INVOKE_FUNCTION")]
    Invoke,
    #[serde(rename = "DECLARE")]
    Declare,
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AddTransactionResponseType {
    Invoke(InvokeResponse),
    Declare(DeclareResponse),
    DeployAccount(DeployAccountResponse),
}

impl fmt::Display for AddTransactionResponseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddTransactionResponseType::Invoke(invoke_response) => {
                write!(
                    f,
                    "INVOKE_FUNCTION - Code: {}, Transaction Hash: {:?}",
                    invoke_response.code, invoke_response.transaction_hash
                )
            }
            AddTransactionResponseType::Declare(declare_response) => {
                write!(
                    f,
                    "DECLARE - Code: {}, Transaction Hash: {:?}, Class Hash: {:?}",
                    declare_response.code,
                    declare_response.transaction_hash,
                    declare_response.class_hash
                )
            }
            AddTransactionResponseType::DeployAccount(deploy_account_response) => {
                write!(
                    f,
                    "DEPLOY_ACCOUNT - Code: {}, Transaction Hash: {:?}",
                    deploy_account_response.code, deploy_account_response.transaction_hash
                )
            }
        }
    }
}
