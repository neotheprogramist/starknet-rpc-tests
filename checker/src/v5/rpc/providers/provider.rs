use auto_impl::auto_impl;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_5_0::{
    AddInvokeTransactionResult, BlockHashAndNumber, BlockId, BroadcastedDeclareTxn,
    BroadcastedDeployAccountTxn, BroadcastedInvokeTxn, BroadcastedTxn, ClassAndTxnHash,
    ContractAndTxnHash, ContractClass, EventFilterWithPageRequest, EventsChunk, FeeEstimate,
    FunctionCall, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs, MaybePendingStateUpdate,
    MsgFromL1, SimulateTransactionsResult, SimulationFlag, SyncingStatus,
    TraceBlockTransactionsResult, TransactionTrace, Txn, TxnFinalityAndExecutionStatus, TxnReceipt,
};

use std::{any::Any, error::Error, fmt::Debug};

use super::jsonrpc::StarknetError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationFlagForEstimateFee {
    #[serde(rename = "SKIP_VALIDATE")]
    SkipValidate,
}

#[auto_impl(&, Box, Arc)]
pub trait Provider {
    /// Returns the version of the Starknet JSON-RPC specification being used
    fn spec_version(&self) -> impl std::future::Future<Output = Result<String, ProviderError>>;

    /// Get block information with transaction hashes given the block id
    fn get_block_with_tx_hashes(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<MaybePendingBlockWithTxHashes, ProviderError>>;

    /// Get block information with full transactions given the block id
    fn get_block_with_txs(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<MaybePendingBlockWithTxs, ProviderError>>;

    /// Get the information about the result of executing the requested block
    fn get_state_update(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<MaybePendingStateUpdate, ProviderError>>;

    /// Get the value of the storage at the given address and key
    fn get_storage_at(
        &self,
        contract_address: Felt,
        key: Felt,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<Felt, ProviderError>>;

    /// Gets the transaction status (possibly reflecting that the tx is still in
    /// the mempool, or dropped from it)
    fn get_transaction_status(
        &self,
        transaction_hash: Felt,
    ) -> impl std::future::Future<Output = Result<TxnFinalityAndExecutionStatus, ProviderError>>;

    /// Get the details and status of a submitted transaction
    fn get_transaction_by_hash(
        &self,
        transaction_hash: Felt,
    ) -> impl std::future::Future<Output = Result<Txn, ProviderError>>;

    /// Get the details of a transaction by a given block id and index
    fn get_transaction_by_block_id_and_index(
        &self,
        block_id: BlockId,
        index: u64,
    ) -> impl std::future::Future<Output = Result<Txn, ProviderError>>;

    /// Get the details of a transaction by a given block number and index
    fn get_transaction_receipt(
        &self,
        transaction_hash: Felt,
    ) -> impl std::future::Future<Output = Result<TxnReceipt, ProviderError>>;

    /// Get the contract class definition in the given block associated with the given hash
    fn get_class(
        &self,
        block_id: BlockId,
        class_hash: Felt,
    ) -> impl std::future::Future<Output = Result<ContractClass, ProviderError>>;

    /// Get the contract class hash in the given block for the contract deployed at the given address
    fn get_class_hash_at(
        &self,
        block_id: BlockId,
        contract_address: Felt,
    ) -> impl std::future::Future<Output = Result<Felt, ProviderError>>;

    /// Get the contract class definition in the given block at the given address
    fn get_class_at(
        &self,
        block_id: BlockId,
        contract_address: Felt,
    ) -> impl std::future::Future<Output = Result<ContractClass, ProviderError>>;

    /// Get the number of transactions in a block given a block id
    fn get_block_transaction_count(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<u64, ProviderError>>;

    /// Call a starknet function without creating a Starknet transaction
    fn call(
        &self,
        request: FunctionCall,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<Vec<Felt>, ProviderError>>;

    /// Estimate the fee for a given Starknet transaction
    fn estimate_fee(
        &self,
        request: Vec<BroadcastedTxn>,
        simulation_flags: Vec<String>,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<Vec<FeeEstimate>, ProviderError>>;

    /// Same as [estimate_fee], but only with one estimate.
    async fn estimate_fee_single(
        &self,
        request: BroadcastedTxn,
        simulation_flags: Vec<String>,
        block_id: BlockId,
    ) -> Result<FeeEstimate, ProviderError> {
        let mut result = self
            .estimate_fee(vec![request], simulation_flags, block_id)
            .await?;

        if result.len() == 1 {
            // Unwrapping here is safe becuase we already checked length
            Ok(result.pop().unwrap())
        } else {
            Err(ProviderError::ArrayLengthMismatch)
        }
    }

    fn estimate_message_fee(
        &self,
        message: MsgFromL1,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<FeeEstimate, ProviderError>>;

    /// Get the most recent accepted block number
    fn block_number(&self) -> impl std::future::Future<Output = Result<u64, ProviderError>>;

    /// Get the most recent accepted block hash and number
    fn block_hash_and_number(
        &self,
    ) -> impl std::future::Future<Output = Result<BlockHashAndNumber, ProviderError>>;

    /// Return the currently configured Starknet chain id
    fn chain_id(&self) -> impl std::future::Future<Output = Result<Felt, ProviderError>>;

    /// Returns an object about the sync status, or false if the node is not synching
    fn syncing(&self) -> impl std::future::Future<Output = Result<SyncingStatus, ProviderError>>;

    /// Returns all events matching the given filter
    fn get_events(
        &self,
        filter: EventFilterWithPageRequest,
    ) -> impl std::future::Future<Output = Result<EventsChunk, ProviderError>>;

    /// Get the nonce associated with the given address in the given block
    fn get_nonce(
        &self,
        block_id: BlockId,
        contract_address: Felt,
    ) -> impl std::future::Future<Output = Result<Felt, ProviderError>>;

    /// Submit a new transaction to be added to the chain
    fn add_invoke_transaction(
        &self,
        invoke_transaction: BroadcastedInvokeTxn,
    ) -> impl std::future::Future<Output = Result<AddInvokeTransactionResult, ProviderError>>;

    /// Submit a new transaction to be added to the chain
    fn add_declare_transaction(
        &self,
        declare_transaction: BroadcastedDeclareTxn,
    ) -> impl std::future::Future<Output = Result<ClassAndTxnHash, ProviderError>>;

    /// Submit a new deploy account transaction
    fn add_deploy_account_transaction(
        &self,
        deploy_account_transaction: BroadcastedDeployAccountTxn,
    ) -> impl std::future::Future<Output = Result<ContractAndTxnHash, ProviderError>>;

    /// For a given executed transaction, return the trace of its execution, including internal
    /// calls
    fn trace_transaction(
        &self,
        transaction_hash: Felt,
    ) -> impl std::future::Future<Output = Result<TransactionTrace, ProviderError>>;

    /// Simulate a given sequence of transactions on the requested state, and generate the execution
    /// traces. Note that some of the transactions may revert, in which case no error is thrown, but
    /// revert details can be seen on the returned trace object. . Note that some of the
    /// transactions may revert, this will be reflected by the revert_error property in the trace.
    /// Other types of failures (e.g. unexpected error or failure in the validation phase) will
    /// result in TRANSACTION_EXECUTION_ERROR.
    fn simulate_transactions(
        &self,
        block_id: BlockId,
        transactions: Vec<BroadcastedTxn>,
        simulation_flags: Vec<SimulationFlag>,
    ) -> impl std::future::Future<Output = Result<Vec<SimulateTransactionsResult>, ProviderError>>;

    /// Retrieve traces for all transactions in the given block.
    fn trace_block_transactions(
        &self,
        block_id: BlockId,
    ) -> impl std::future::Future<Output = Result<Vec<TraceBlockTransactionsResult>, ProviderError>>;

    /// Same as [simulate_transactions], but only with one simulation.
    async fn simulate_transaction(
        &self,
        block_id: BlockId,
        transaction: BroadcastedTxn,
        simulation_flags: Vec<SimulationFlag>,
    ) -> Result<SimulateTransactionsResult, ProviderError> {
        let mut result = self
            .simulate_transactions(block_id, vec![transaction], simulation_flags)
            .await?;

        if result.len() == 1 {
            // Unwrapping here is safe becuase we already checked length
            Ok(result.pop().unwrap())
        } else {
            Err(ProviderError::ArrayLengthMismatch)
        }
    }
}

/// Trait for implementation-specific error type. These errors are irrelevant in most cases,
/// assuming that users typically care more about the specifics of RPC errors instead of the
/// underlying transport. Therefore, it makes little sense to bloat [ProviderError] with a generic
/// parameter just for these errors. Instead, they're erased to this trait object.
///
/// This trait is used instead of a plain [std::error::Error] to allow downcasting, in case access
/// to the specific error type is indeed desired. This is achieved with the `as_any()` method.
#[allow(dead_code)]
pub trait ProviderImplError: Error + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ProviderError {
    #[error(transparent)]
    StarknetError(StarknetError),
    #[error("Request rate limited")]
    RateLimited,
    #[error("Array length mismatch")]
    ArrayLengthMismatch,
    #[error("{0}")]
    Other(Box<dyn ProviderImplError>),
}
