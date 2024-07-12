use auto_impl::auto_impl;
use starknet_types_rpc::{
    AddInvokeTransactionResult, BlockHashAndNumber, BlockId, BroadcastedDeclareTxn,
    BroadcastedDeployAccountTxn, BroadcastedInvokeTxn, BroadcastedTxn, ClassAndTxnHash,
    ContractAndTxnHash, ContractClass, EventFilterWithPageRequest, EventsChunk, FeeEstimate, Felt,
    FunctionCall, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs, MaybePendingStateUpdate,
    MsgFromL1, SimulateTransactionsResult, SimulationFlag, SyncingStatus,
    TraceBlockTransactionsResult, TransactionTrace, Txn, TxnReceipt, TxnStatus,
};
use std::{any::Any, error::Error, fmt::Debug};

use super::jsonrpc::StarknetError;

#[auto_impl(&, Box, Arc)]
pub trait Provider {
    /// Returns the version of the Starknet JSON-RPC specification being used
    fn spec_version(&self) -> impl std::future::Future<Output = Result<String, ProviderError>>;

    /// Get block information with transaction hashes given the block id
    fn get_block_with_tx_hashes<B>(
        &self,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<MaybePendingBlockWithTxHashes, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get block information with full transactions given the block id
    fn get_block_with_txs<B>(
        &self,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<MaybePendingBlockWithTxs, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get the information about the result of executing the requested block
    fn get_state_update<B>(
        &self,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<MaybePendingStateUpdate, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get the value of the storage at the given address and key
    fn get_storage_at<A, K, B>(
        &self,
        contract_address: A,
        key: K,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<Felt, ProviderError>>
    where
        A: AsRef<Felt> + Send + Sync,
        K: AsRef<Felt> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync;

    /// Gets the transaction status (possibly reflecting that the tx is still in
    /// the mempool, or dropped from it)
    fn get_transaction_status<H>(
        &self,
        transaction_hash: H,
    ) -> impl std::future::Future<Output = Result<TxnStatus, ProviderError>>
    where
        H: AsRef<Felt> + Send + Sync;

    /// Get the details and status of a submitted transaction
    fn get_transaction_by_hash<H>(
        &self,
        transaction_hash: H,
    ) -> impl std::future::Future<Output = Result<Txn, ProviderError>>
    where
        H: AsRef<Felt> + Send + Sync;

    /// Get the details of a transaction by a given block id and index
    fn get_transaction_by_block_id_and_index<B>(
        &self,
        block_id: B,
        index: u64,
    ) -> impl std::future::Future<Output = Result<Txn, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get the details of a transaction by a given block number and index
    fn get_transaction_receipt<H>(
        &self,
        transaction_hash: H,
    ) -> impl std::future::Future<Output = Result<TxnReceipt, ProviderError>>
    where
        H: AsRef<Felt> + Send + Sync;

    /// Get the contract class definition in the given block associated with the given hash
    fn get_class<B, H>(
        &self,
        block_id: B,
        class_hash: H,
    ) -> impl std::future::Future<Output = Result<ContractClass, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync,
        H: AsRef<Felt> + Send + Sync;

    /// Get the contract class hash in the given block for the contract deployed at the given address
    fn get_class_hash_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> impl std::future::Future<Output = Result<Felt, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<Felt> + Send + Sync;

    /// Get the contract class definition in the given block at the given address
    fn get_class_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> impl std::future::Future<Output = Result<ContractClass, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<Felt> + Send + Sync;

    /// Get the number of transactions in a block given a block id
    fn get_block_transaction_count<B>(
        &self,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<u64, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Call a starknet function without creating a Starknet transaction
    fn call<R, B>(
        &self,
        request: R,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<Vec<Felt>, ProviderError>>
    where
        R: AsRef<FunctionCall> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync;

    /// Estimate the fee for a given Starknet transaction
    fn estimate_fee<R, S, B>(
        &self,
        request: R,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<Vec<FeeEstimate>, ProviderError>>
    where
        R: AsRef<[BroadcastedTxn]> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync;

    fn estimate_message_fee<M, B>(
        &self,
        message: M,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<FeeEstimate, ProviderError>>
    where
        M: AsRef<MsgFromL1> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync;

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
    fn get_nonce<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> impl std::future::Future<Output = Result<Felt, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<Felt> + Send + Sync;

    /// Submit a new transaction to be added to the chain
    fn add_invoke_transaction<I>(
        &self,
        invoke_transaction: I,
    ) -> impl std::future::Future<Output = Result<AddInvokeTransactionResult, ProviderError>>
    where
        I: AsRef<BroadcastedInvokeTxn> + Send + Sync;

    /// Submit a new transaction to be added to the chain
    fn add_declare_transaction<D>(
        &self,
        declare_transaction: D,
    ) -> impl std::future::Future<Output = Result<ClassAndTxnHash, ProviderError>>
    where
        D: AsRef<BroadcastedDeclareTxn> + Send + Sync;

    /// Submit a new deploy account transaction
    fn add_deploy_account_transaction<D>(
        &self,
        deploy_account_transaction: D,
    ) -> impl std::future::Future<Output = Result<ContractAndTxnHash, ProviderError>>
    where
        D: AsRef<BroadcastedDeployAccountTxn> + Send + Sync;

    /// For a given executed transaction, return the trace of its execution, including internal
    /// calls
    fn trace_transaction<H>(
        &self,
        transaction_hash: H,
    ) -> impl std::future::Future<Output = Result<TransactionTrace, ProviderError>>
    where
        H: AsRef<Felt> + Send + Sync;

    /// Simulate a given sequence of transactions on the requested state, and generate the execution
    /// traces. Note that some of the transactions may revert, in which case no error is thrown, but
    /// revert details can be seen on the returned trace object. . Note that some of the
    /// transactions may revert, this will be reflected by the revert_error property in the trace.
    /// Other types of failures (e.g. unexpected error or failure in the validation phase) will
    /// result in TRANSACTION_EXECUTION_ERROR.
    fn simulate_transactions<B, T, S>(
        &self,
        block_id: B,
        transactions: T,
        simulation_flags: S,
    ) -> impl std::future::Future<Output = Result<Vec<SimulateTransactionsResult>, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync,
        T: AsRef<[BroadcastedTxn]> + Send + Sync,
        S: AsRef<[SimulationFlag]> + Send + Sync;

    /// Retrieve traces for all transactions in the given block.
    fn trace_block_transactions<B>(
        &self,
        block_id: B,
    ) -> impl std::future::Future<Output = Result<Vec<TraceBlockTransactionsResult>, ProviderError>>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Same as [simulate_transactions], but only with one simulation.
    async fn simulate_transaction<B, T, S>(
        &self,
        block_id: B,
        transaction: T,
        simulation_flags: S,
    ) -> Result<SimulateTransactionsResult, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        T: AsRef<BroadcastedTxn> + Send + Sync,
        S: AsRef<[SimulationFlag]> + Send + Sync,
    {
        let mut result = self
            .simulate_transactions(
                block_id,
                [transaction.as_ref().to_owned()],
                simulation_flags,
            )
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
pub trait ProviderImplError: Error + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, thiserror::Error)]
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
