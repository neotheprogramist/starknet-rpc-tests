use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet_crypto::FieldElement;
use std::{any::Any, error::Error, fmt::Debug};

use crate::{
    codegen::{
        BlockTag, FeeEstimate, FunctionCall, SimulatedTransaction, SimulationFlag,
        SimulationFlagForEstimateFee, TransactionReceiptWithBlockInfo,
    },
    models::{
        BroadcastedTransaction, ContractClass, DeclareTransactionResult, FeeUnit,
        InvokeTransactionResult, MaybePendingBlockWithReceipts, MaybePendingBlockWithTxs,
        MaybePendingStateUpdate, Transaction, TransactionStatus,
    },
    transports::MaybePendingBlockWithTxHashes,
};

use super::{
    codegen::StarknetError,
    models::{BlockId, BroadcastedDeclareTransaction, BroadcastedInvokeTransaction},
};
#[allow(async_fn_in_trait)]
pub trait Provider {
    /// Returns the version of the Starknet JSON-RPC specification being used
    async fn spec_version(&self) -> Result<String, ProviderError>;

    /// Get block information with transaction hashes given the block id
    async fn get_block_with_tx_hashes<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithTxHashes, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get block information with full transactions given the block id
    async fn get_block_with_txs<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithTxs, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get block information with full transactions and receipts given the block id
    async fn get_block_with_receipts<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithReceipts, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get the information about the result of executing the requested block
    async fn get_state_update<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingStateUpdate, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync;

    /// Get the value of the storage at the given address and key
    async fn get_storage_at<A, K, B>(
        &self,
        contract_address: A,
        key: K,
        block_id: B,
    ) -> Result<FieldElement, ProviderError>
    where
        A: AsRef<FieldElement> + Send + Sync,
        K: AsRef<FieldElement> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync;

    /// Gets the transaction status (possibly reflecting that the tx is still in
    /// the mempool, or dropped from it)
    async fn get_transaction_status<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionStatus, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync;

    /// Get the details and status of a submitted transaction
    async fn get_transaction_by_hash<H>(
        &self,
        transaction_hash: H,
    ) -> Result<Transaction, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync;

    // /// Get the details of a transaction by a given block id and index
    // async fn get_transaction_by_block_id_and_index<B>(
    //     &self,
    //     block_id: B,
    //     index: u64,
    // ) -> Result<Transaction, ProviderError>
    // where
    //     B: AsRef<BlockId> + Send + Sync;

    /// Get the details of a transaction by a given block number and index
    async fn get_transaction_receipt<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionReceiptWithBlockInfo, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync;

    /// Get the contract class definition in the given block associated with the given hash
    async fn get_class<B, H>(
        &self,
        block_id: B,
        class_hash: H,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        H: AsRef<FieldElement> + Send + Sync;

    /// Get the contract class hash in the given block for the contract deployed at the given address
    async fn get_class_hash_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FieldElement, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync;

    // /// Get the contract class definition in the given block at the given address
    // async fn get_class_at<B, A>(
    //     &self,
    //     block_id: B,
    //     contract_address: A,
    // ) -> Result<ContractClass, ProviderError>
    // where
    //     B: AsRef<BlockId> + Send + Sync,
    //     A: AsRef<FieldElement> + Send + Sync;

    // /// Get the number of transactions in a block given a block id
    // async fn get_block_transaction_count<B>(&self, block_id: B) -> Result<u64, ProviderError>
    // where
    //     B: AsRef<BlockId> + Send + Sync;

    /// Call a starknet function without creating a Starknet transaction
    async fn call<R, B>(&self, request: R, block_id: B) -> Result<Vec<FieldElement>, ProviderError>
    where
        R: AsRef<FunctionCall> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync;

    /// Estimate the fee for a given Starknet transaction
    async fn estimate_fee<R, S, B>(
        &self,
        request: R,
        simulation_flags: S,
        block_id: B,
    ) -> Result<Vec<FeeEstimate>, ProviderError>
    where
        R: AsRef<[BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[SimulationFlagForEstimateFee]> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync;

    // async fn estimate_message_fee<M, B>(
    //     &self,
    //     message: M,
    //     block_id: B,
    // ) -> Result<FeeEstimate, ProviderError>
    // where
    //     M: AsRef<MsgFromL1> + Send + Sync,
    //     B: AsRef<BlockId> + Send + Sync;

    // /// Get the most recent accepted block number
    // async fn block_number(&self) -> Result<u64, ProviderError>;

    // /// Get the most recent accepted block hash and number
    // async fn block_hash_and_number(&self) -> Result<BlockHashAndNumber, ProviderError>;

    // /// Return the currently configured Starknet chain id
    // async fn chain_id(&self) -> Result<FieldElement, ProviderError>;

    // /// Returns an object about the sync status, or false if the node is not synching
    // async fn syncing(&self) -> Result<SyncStatusType, ProviderError>;

    // /// Returns all events matching the given filter
    // async fn get_events(
    //     &self,
    //     filter: EventFilter,
    //     continuation_token: Option<String>,
    //     chunk_size: u64,
    // ) -> Result<EventsPage, ProviderError>;

    /// Get the nonce associated with the given address in the given block
    async fn get_nonce<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FieldElement, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync;

    /// Submit a new transaction to be added to the chain
    async fn add_invoke_transaction<I>(
        &self,
        invoke_transaction: I,
    ) -> Result<InvokeTransactionResult, ProviderError>
    where
        I: AsRef<BroadcastedInvokeTransaction> + Send + Sync;

    /// Submit a new transaction to be added to the chain
    async fn add_declare_transaction<D>(
        &self,
        declare_transaction: D,
    ) -> Result<DeclareTransactionResult, ProviderError>
    where
        D: AsRef<BroadcastedDeclareTransaction> + Send + Sync;

    // /// Submit a new deploy account transaction
    // async fn add_deploy_account_transaction<D>(
    //     &self,
    //     deploy_account_transaction: D,
    // ) -> Result<DeployAccountTransactionResult, ProviderError>
    // where
    //     D: AsRef<BroadcastedDeployAccountTransaction> + Send + Sync;

    // /// For a given executed transaction, return the trace of its execution, including internal
    // /// calls
    // async fn trace_transaction<H>(
    //     &self,
    //     transaction_hash: H,
    // ) -> Result<TransactionTrace, ProviderError>
    // where
    //     H: AsRef<FieldElement> + Send + Sync;

    /// Simulate a given sequence of transactions on the requested state, and generate the execution
    /// traces. Note that some of the transactions may revert, in which case no error is thrown, but
    /// revert details can be seen on the returned trace object. . Note that some of the
    /// transactions may revert, this will be reflected by the revert_error property in the trace.
    /// Other types of failures (e.g. unexpected error or failure in the validation phase) will
    /// result in TRANSACTION_EXECUTION_ERROR.
    async fn simulate_transactions<B, T, S>(
        &self,
        block_id: B,
        transactions: T,
        simulation_flags: S,
    ) -> Result<Vec<SimulatedTransaction>, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        T: AsRef<[BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[SimulationFlag]> + Send + Sync;

    // /// Retrieve traces for all transactions in the given block.
    // async fn trace_block_transactions<B>(
    //     &self,
    //     block_id: B,
    // ) -> Result<Vec<TransactionTraceWithHash>, ProviderError>
    // where
    //     B: AsRef<BlockId> + Send + Sync;

    /// Same as [estimate_fee], but only with one estimate.
    async fn estimate_fee_single<R, S, B>(
        &self,
        request: R,
        simulation_flags: S,
        block_id: B,
    ) -> Result<FeeEstimate, ProviderError>
    where
        R: AsRef<BroadcastedTransaction> + Send + Sync,
        S: AsRef<[SimulationFlagForEstimateFee]> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        let mut result = self
            .estimate_fee([request.as_ref().to_owned()], simulation_flags, block_id)
            .await?;

        if result.len() == 1 {
            // Unwrapping here is safe becuase we already checked length
            Ok(result.pop().unwrap())
        } else {
            Err(ProviderError::ArrayLengthMismatch)
        }
    }

    /// Same as [simulate_transactions], but only with one simulation.
    async fn simulate_transaction<B, T, S>(
        &self,
        block_id: B,
        transaction: T,
        simulation_flags: S,
    ) -> Result<SimulatedTransaction, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        T: AsRef<BroadcastedTransaction> + Send + Sync,
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
    async fn set_time(&self, time: u64, generate_block: bool) -> Result<Value, ProviderError>;
    async fn increase_time(&self, increase_time: u64) -> Result<Value, ProviderError>;
    async fn create_block(&self) -> Result<Value, ProviderError>;
    async fn abort_blocks(&self, starting_block_hash: String) -> Result<Value, ProviderError>;
    async fn get_config(&self) -> Result<Value, ProviderError>;
    async fn get_predeployed_accounts(&self) -> Result<Value, ProviderError>;
    async fn mint(&self, address: FieldElement, mint_amount: u128) -> Result<Value, ProviderError>;
    async fn get_account_balance(
        &self,
        address: FieldElement,
        unit: FeeUnit,
        block_tag: BlockTag,
    ) -> Result<Value, ProviderError>;
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    seed: u64,
    total_accounts: u32,
    account_contract_class_hash: String,
    predeployed_accounts_initial_balance: String,
    start_time: Option<String>,
    gas_price_wei: u64,
    gas_price_strk: u64,
    data_gas_price_wei: u64,
    data_gas_price_strk: u64,
    chain_id: String,
    dump_on: String,
    dump_path: String,
    state_archive: String,
    fork_config: ForkConfig,
    server_config: ServerConfig,
    blocks_on_demand: bool,
    lite_mode: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForkConfig {
    url: String,
    block_number: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    host: String,
    port: u16,
    timeout: u64,
    request_body_size_limit: u64,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
