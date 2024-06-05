use ::serde::Deserialize;
use codegen::BroadcastedDeclareTransactionV3;
use serde::Serialize;
pub mod byte_array;
pub mod codegen;
pub mod serde_impls;
pub mod transaction_request;
pub mod unsigned_field_element;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum BroadcastedTransaction {
    // #[serde(rename = "INVOKE")]
    // Invoke(BroadcastedInvokeTransaction),
    #[serde(rename = "DECLARE")]
    Declare(BroadcastedDeclareTransaction),
    // #[serde(rename = "DEPLOY_ACCOUNT")]
    // DeployAccount(BroadcastedDeployAccountTransaction),
}
impl AsRef<BroadcastedTransaction> for BroadcastedTransaction {
    fn as_ref(&self) -> &BroadcastedTransaction {
        self
    }
}

impl AsRef<BroadcastedDeclareTransaction> for BroadcastedDeclareTransaction {
    fn as_ref(&self) -> &BroadcastedDeclareTransaction {
        self
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BroadcastedDeclareTransaction {
    // V1(BroadcastedDeclareTransactionV1),
    // V2(BroadcastedDeclareTransactionV2),
    V3(BroadcastedDeclareTransactionV3),
}
