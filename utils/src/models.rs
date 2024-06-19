use crate::codegen::{
    BlockTag, BlockWithReceipts, BlockWithTxs, BroadcastedDeclareTransactionV1,
    BroadcastedDeclareTransactionV2, BroadcastedDeclareTransactionV3,
    BroadcastedDeployAccountTransactionV1, BroadcastedDeployAccountTransactionV3,
    BroadcastedInvokeTransactionV1, BroadcastedInvokeTransactionV3, DeclareTransactionReceipt,
    DeclareTransactionTrace, DeclareTransactionV0, DeclareTransactionV1, DeclareTransactionV2,
    DeclareTransactionV3, DeployAccountTransactionReceipt, DeployAccountTransactionTrace,
    DeployAccountTransactionV1, DeployAccountTransactionV3, DeployTransaction,
    DeployTransactionReceipt, FunctionCall, InvokeTransactionReceipt, InvokeTransactionTrace,
    InvokeTransactionV0, InvokeTransactionV1, InvokeTransactionV3, L1HandlerTransaction,
    L1HandlerTransactionReceipt, L1HandlerTransactionTrace, PendingBlockWithReceipts,
    PendingBlockWithTxs, PendingStateUpdate, ResourcePrice, SequencerTransactionStatus,
    StateUpdate, TransactionExecutionStatus, TransactionWithReceipt,
};
use crate::unsigned_field_element::UfeHex;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet_crypto::FieldElement;
#[allow(dead_code)]
/// Cairo string for "CONTRACT_CLASS_V0.1.0"
const PREFIX_CONTRACT_CLASS_V0_1_0: FieldElement = FieldElement::from_mont([
    5800711240972404213,
    15539482671244488427,
    18446734822722598327,
    37302452645455172,
]);

#[allow(dead_code)]
/// Cairo string for "COMPILED_CLASS_V1"
const PREFIX_COMPILED_CLASS_V1: FieldElement = FieldElement::from_mont([
    2291010424822318237,
    1609463842841646376,
    18446744073709549462,
    324306817650036332,
]);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum BroadcastedTransaction {
    #[serde(rename = "INVOKE")]
    Invoke(BroadcastedInvokeTransaction),
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

/// Block hash, number or tag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockId {
    Hash(FieldElement),
    Number(u64),
    Tag(BlockTag),
}

impl AsRef<BlockId> for BlockId {
    fn as_ref(&self) -> &BlockId {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum BroadcastedInvokeTransaction {
    V1(BroadcastedInvokeTransactionV1),
    V3(BroadcastedInvokeTransactionV3),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum BroadcastedDeclareTransaction {
    V1(BroadcastedDeclareTransactionV1),
    V2(BroadcastedDeclareTransactionV2),
    V3(BroadcastedDeclareTransactionV3),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum BroadcastedDeployAccountTransaction {
    V1(BroadcastedDeployAccountTransactionV1),
    V3(BroadcastedDeployAccountTransactionV3),
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvokeTransactionResult {
    /// The hash of the invoke transaction
    #[serde_as(as = "UfeHex")]
    pub transaction_hash: FieldElement,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclareTransactionResult {
    /// The hash of the declare transaction
    #[serde_as(as = "UfeHex")]
    pub transaction_hash: FieldElement,
    /// The hash of the declared class
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployTransactionResult {
    /// The hash of the deploy transaction
    #[serde_as(as = "UfeHex")]
    pub transaction_hash: FieldElement,
    /// The address of the new contract
    #[serde_as(as = "UfeHex")]
    pub contract_address: FieldElement,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "INVOKE")]
    Invoke(InvokeTransaction),
    #[serde(rename = "L1_HANDLER")]
    L1Handler(L1HandlerTransaction),
    #[serde(rename = "DECLARE")]
    Declare(DeclareTransaction),
    #[serde(rename = "DEPLOY")]
    Deploy(DeployTransaction),
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeployAccountTransaction),
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "version")]
pub enum DeployAccountTransaction {
    #[serde(rename = "0x1")]
    V1(DeployAccountTransactionV1),
    #[serde(rename = "0x3")]
    V3(DeployAccountTransactionV3),
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "version")]
pub enum InvokeTransaction {
    #[serde(rename = "0x0")]
    V0(InvokeTransactionV0),
    #[serde(rename = "0x1")]
    V1(InvokeTransactionV1),
    #[serde(rename = "0x3")]
    V3(InvokeTransactionV3),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "version")]
pub enum DeclareTransaction {
    #[serde(rename = "0x0")]
    V0(DeclareTransactionV0),
    #[serde(rename = "0x1")]
    V1(DeclareTransactionV1),
    #[serde(rename = "0x2")]
    V2(DeclareTransactionV2),
    #[serde(rename = "0x3")]
    V3(DeclareTransactionV3),
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionTrace {
    #[serde(rename = "INVOKE")]
    Invoke(InvokeTransactionTrace),
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeployAccountTransactionTrace),
    #[serde(rename = "L1_HANDLER")]
    L1Handler(L1HandlerTransactionTrace),
    #[serde(rename = "DECLARE")]
    Declare(DeclareTransactionTrace),
}
impl AsRef<BroadcastedInvokeTransaction> for BroadcastedInvokeTransaction {
    fn as_ref(&self) -> &BroadcastedInvokeTransaction {
        self
    }
}
impl AsRef<FunctionCall> for FunctionCall {
    fn as_ref(&self) -> &FunctionCall {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionReceipt {
    #[serde(rename = "INVOKE")]
    Invoke(InvokeTransactionReceipt),
    #[serde(rename = "L1_HANDLER")]
    L1Handler(L1HandlerTransactionReceipt),
    #[serde(rename = "DECLARE")]
    Declare(DeclareTransactionReceipt),
    #[serde(rename = "DEPLOY")]
    Deploy(DeployTransactionReceipt),
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeployAccountTransactionReceipt),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybePendingBlockWithTxs {
    Block(BlockWithTxs),
    PendingBlock(PendingBlockWithTxs),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybePendingBlockWithReceipts {
    Block(BlockWithReceipts),
    PendingBlock(PendingBlockWithReceipts),
}
impl MaybePendingBlockWithReceipts {
    pub fn transactions(&self) -> &[TransactionWithReceipt] {
        match self {
            MaybePendingBlockWithReceipts::Block(block) => &block.transactions,
            MaybePendingBlockWithReceipts::PendingBlock(block) => &block.transactions,
        }
    }

    pub fn l1_gas_price(&self) -> &ResourcePrice {
        match self {
            MaybePendingBlockWithReceipts::Block(block) => &block.l1_gas_price,
            MaybePendingBlockWithReceipts::PendingBlock(block) => &block.l1_gas_price,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybePendingStateUpdate {
    Update(StateUpdate),
    PendingUpdate(PendingStateUpdate),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    Received,
    Rejected,
    AcceptedOnL2(TransactionExecutionStatus),
    AcceptedOnL1(TransactionExecutionStatus),
}
impl TransactionStatus {
    pub fn finality_status(&self) -> SequencerTransactionStatus {
        match self {
            TransactionStatus::Received => SequencerTransactionStatus::Received,
            TransactionStatus::Rejected => SequencerTransactionStatus::Rejected,
            TransactionStatus::AcceptedOnL2(_) => SequencerTransactionStatus::AcceptedOnL2,
            TransactionStatus::AcceptedOnL1(_) => SequencerTransactionStatus::AcceptedOnL1,
        }
    }
}
impl DeployAccountTransaction {
    pub fn transaction_hash(&self) -> &FieldElement {
        match self {
            DeployAccountTransaction::V1(tx) => &tx.transaction_hash,
            DeployAccountTransaction::V3(tx) => &tx.transaction_hash,
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum FeeUnit {
    WEI,
    FRI,
}

impl std::fmt::Display for FeeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FeeUnit::WEI => "WEI",
            FeeUnit::FRI => "FRI",
        })
    }
}
