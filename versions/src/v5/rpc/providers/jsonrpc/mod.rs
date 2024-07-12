pub mod transports;
use std::{any::Any, error::Error, fmt::Display};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use starknet_types_rpc::{
    AddDeclareTransactionParams, AddDeployAccountTransactionParams, AddInvokeTransactionParams,
    AddInvokeTransactionResult, BlockHashAndNumber, BlockHashAndNumberParams, BlockId,
    BlockNumberParams, BroadcastedDeclareTxn, BroadcastedDeployAccountTxn, BroadcastedInvokeTxn,
    BroadcastedTxn, CallParams, ChainIdParams, ClassAndTxnHash, ContractAndTxnHash, ContractClass,
    EstimateFeeParams, EstimateMessageFeeParams, EventFilterWithPageRequest, EventsChunk,
    FeeEstimate, Felt as FeltPrimitive, FunctionCall, GetBlockTransactionCountParams,
    GetBlockWithTxHashesParams, GetBlockWithTxsParams, GetClassAtParams, GetClassHashAtParams,
    GetClassParams, GetEventsParams, GetNonceParams, GetStateUpdateParams, GetStorageAtParams,
    GetTransactionByBlockIdAndIndexParams, GetTransactionByHashParams, GetTransactionReceiptParams,
    GetTransactionStatusParams, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs,
    MaybePendingStateUpdate, MsgFromL1, SimulateTransactionsParams, SimulateTransactionsResult,
    SimulationFlag, SpecVersionParams, SyncingParams, SyncingStatus, TraceBlockTransactionsParams,
    TraceBlockTransactionsResult, TraceTransactionParams, TransactionTrace, Txn, TxnReceipt,
    TxnStatus,
};

pub use transports::{HttpTransport, HttpTransportError, JsonRpcTransport};

use super::provider::{Provider, ProviderError, ProviderImplError};

#[derive(Debug)]
pub struct JsonRpcClient<T> {
    transport: T,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JsonRpcMethod {
    #[serde(rename = "starknet_specVersion")]
    SpecVersion,
    #[serde(rename = "starknet_getBlockWithTxHashes")]
    GetBlockWithTxHashes,
    #[serde(rename = "starknet_getBlockWithTxs")]
    GetBlockWithTxs,
    #[serde(rename = "starknet_getStateUpdate")]
    GetStateUpdate,
    #[serde(rename = "starknet_getStorageAt")]
    GetStorageAt,
    #[serde(rename = "starknet_getTransactionStatus")]
    GetTransactionStatus,
    #[serde(rename = "starknet_getTransactionByHash")]
    GetTransactionByHash,
    #[serde(rename = "starknet_getTransactionByBlockIdAndIndex")]
    GetTransactionByBlockIdAndIndex,
    #[serde(rename = "starknet_getTransactionReceipt")]
    GetTransactionReceipt,
    #[serde(rename = "starknet_getClass")]
    GetClass,
    #[serde(rename = "starknet_getClassHashAt")]
    GetClassHashAt,
    #[serde(rename = "starknet_getClassAt")]
    GetClassAt,
    #[serde(rename = "starknet_getBlockTransactionCount")]
    GetBlockTransactionCount,
    #[serde(rename = "starknet_call")]
    Call,
    #[serde(rename = "starknet_estimateFee")]
    EstimateFee,
    #[serde(rename = "starknet_estimateMessageFee")]
    EstimateMessageFee,
    #[serde(rename = "starknet_blockNumber")]
    BlockNumber,
    #[serde(rename = "starknet_blockHashAndNumber")]
    BlockHashAndNumber,
    #[serde(rename = "starknet_chainId")]
    ChainId,
    #[serde(rename = "starknet_syncing")]
    Syncing,
    #[serde(rename = "starknet_getEvents")]
    GetEvents,
    #[serde(rename = "starknet_getNonce")]
    GetNonce,
    #[serde(rename = "starknet_addInvokeTransaction")]
    AddInvokeTransaction,
    #[serde(rename = "starknet_addDeclareTransaction")]
    AddDeclareTransaction,
    #[serde(rename = "starknet_addDeployAccountTransaction")]
    AddDeployAccountTransaction,
    #[serde(rename = "starknet_traceTransaction")]
    TraceTransaction,
    #[serde(rename = "starknet_simulateTransactions")]
    SimulateTransactions,
    #[serde(rename = "starknet_traceBlockTransactions")]
    TraceBlockTransactions,
}

#[derive(Debug, Clone)]
pub struct JsonRpcRequest {
    pub id: u64,
    pub data: JsonRpcRequestData,
}

#[derive(Debug, Clone)]
pub enum JsonRpcRequestData {
    SpecVersion(SpecVersionParams),
    GetBlockWithTxHashes(GetBlockWithTxHashesParams),
    GetBlockWithTxs(GetBlockWithTxsParams),
    GetStateUpdate(GetStateUpdateParams),
    GetStorageAt(GetStorageAtParams),
    GetTransactionStatus(GetTransactionStatusParams),
    GetTransactionByHash(GetTransactionByHashParams),
    GetTransactionByBlockIdAndIndex(GetTransactionByBlockIdAndIndexParams),
    GetTransactionReceipt(GetTransactionReceiptParams),
    GetClass(GetClassParams),
    GetClassHashAt(GetClassHashAtParams),
    GetClassAt(GetClassAtParams),
    GetBlockTransactionCount(GetBlockTransactionCountParams),
    Call(CallParams),
    EstimateFee(EstimateFeeParams),
    EstimateMessageFee(EstimateMessageFeeParams),
    BlockNumber(BlockNumberParams),
    BlockHashAndNumber(BlockHashAndNumberParams),
    ChainId(ChainIdParams),
    Syncing(SyncingParams),
    GetEvents(GetEventsParams),
    GetNonce(GetNonceParams),
    AddInvokeTransaction(AddInvokeTransactionParams),
    AddDeclareTransaction(AddDeclareTransactionParams),
    AddDeployAccountTransaction(AddDeployAccountTransactionParams),
    TraceTransaction(TraceTransactionParams),
    SimulateTransactions(SimulateTransactionsParams),
    TraceBlockTransactions(TraceBlockTransactionsParams),
}

#[derive(Debug, thiserror::Error)]
pub enum JsonRpcClientError<T> {
    #[error(transparent)]
    JsonError(serde_json::Error),
    #[error(transparent)]
    TransportError(T),
    #[error(transparent)]
    JsonRpcError(JsonRpcError),
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse<T> {
    Success { id: u64, result: T },
    Error { id: u64, error: JsonRpcError },
}

/// Failures trying to parse a [JsonRpcError] into [StarknetError].
#[derive(Debug, thiserror::Error)]
pub enum JsonRpcErrorConversionError {
    #[error("unknown error code")]
    UnknownCode,
    #[error("missing data field")]
    MissingData,
    #[error("unable to parse the data field")]
    DataParsingFailure,
}

#[derive(Serialize, Deserialize)]
struct Felt(pub FeltPrimitive);

#[derive(Serialize, Deserialize)]
struct FeltArray(pub Vec<FeltPrimitive>);

impl<T> JsonRpcClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl<T> JsonRpcClient<T>
where
    T: 'static + JsonRpcTransport + Send + Sync,
{
    async fn send_request<P, R>(&self, method: JsonRpcMethod, params: P) -> Result<R, ProviderError>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        match self
            .transport
            .send_request(method, params)
            .await
            .map_err(JsonRpcClientError::TransportError)?
        {
            JsonRpcResponse::Success { result, .. } => Ok(result),
            JsonRpcResponse::Error { error, .. } => {
                Err(match TryInto::<StarknetError>::try_into(&error) {
                    Ok(error) => ProviderError::StarknetError(error),
                    Err(_) => JsonRpcClientError::<T::Error>::JsonRpcError(error).into(),
                })
            }
        }
    }
}

impl<T> Provider for JsonRpcClient<T>
where
    T: 'static + JsonRpcTransport + Sync + Send,
{
    /// Returns the version of the Starknet JSON-RPC specification being used
    async fn spec_version(&self) -> Result<String, ProviderError> {
        self.send_request(JsonRpcMethod::SpecVersion, SpecVersionParams {})
            .await
    }

    /// Get block information with transaction hashes given the block id
    async fn get_block_with_tx_hashes<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithTxHashes, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetBlockWithTxHashes,
            GetBlockWithTxHashesParams {
                block_id: *block_id.as_ref(),
            },
        )
        .await
    }

    /// Get block information with full transactions given the block id
    async fn get_block_with_txs<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithTxs, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetBlockWithTxs,
            GetBlockWithTxsParams {
                block_id: *block_id.as_ref(),
            },
        )
        .await
    }

    /// Get the information about the result of executing the requested block
    async fn get_state_update<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingStateUpdate, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetStateUpdate,
            GetStateUpdateParams {
                block_id: *block_id.as_ref(),
            },
        )
        .await
    }

    /// Get the value of the storage at the given address and key
    async fn get_storage_at<A, K, B>(
        &self,
        contract_address: A,
        key: K,
        block_id: B,
    ) -> Result<FeltPrimitive, ProviderError>
    where
        A: AsRef<FeltPrimitive> + Send + Sync,
        K: AsRef<FeltPrimitive> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        Ok(self
            .send_request::<_, Felt>(
                JsonRpcMethod::GetStorageAt,
                GetStorageAtParams {
                    contract_address: *contract_address.as_ref(),
                    key: key.as_ref().to_string(),
                    block_id: *block_id.as_ref(),
                },
            )
            .await?
            .0)
    }

    /// Gets the transaction status (possibly reflecting that the tx is still in
    /// the mempool, or dropped from it)
    async fn get_transaction_status<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TxnStatus, ProviderError>
    where
        H: AsRef<FeltPrimitive> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetTransactionStatus,
            GetTransactionStatusParams {
                transaction_hash: *transaction_hash.as_ref(),
            },
        )
        .await
    }

    /// Get the details and status of a submitted transaction
    async fn get_transaction_by_hash<H>(&self, transaction_hash: H) -> Result<Txn, ProviderError>
    where
        H: AsRef<FeltPrimitive> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetTransactionByHash,
            GetTransactionByHashParams {
                transaction_hash: *transaction_hash.as_ref(),
            },
        )
        .await
    }

    /// Get the details of a transaction by a given block id and index
    async fn get_transaction_by_block_id_and_index<B>(
        &self,
        block_id: B,
        index: u64,
    ) -> Result<Txn, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetTransactionByBlockIdAndIndex,
            GetTransactionByBlockIdAndIndexParams {
                block_id: *block_id.as_ref(),
                index,
            },
        )
        .await
    }

    /// Get the details of a transaction by a given block number and index
    async fn get_transaction_receipt<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TxnReceipt, ProviderError>
    where
        H: AsRef<FeltPrimitive> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetTransactionReceipt,
            GetTransactionReceiptParams {
                transaction_hash: *transaction_hash.as_ref(),
            },
        )
        .await
    }

    /// Get the contract class definition in the given block associated with the given hash
    async fn get_class<B, H>(
        &self,
        block_id: B,
        class_hash: H,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        H: AsRef<FeltPrimitive> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetClass,
            GetClassParams {
                block_id: *block_id.as_ref(),
                class_hash: *class_hash.as_ref(),
            },
        )
        .await
    }

    /// Get the contract class hash in the given block for the contract deployed at the given address
    async fn get_class_hash_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FeltPrimitive, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FeltPrimitive> + Send + Sync,
    {
        Ok(self
            .send_request::<_, Felt>(
                JsonRpcMethod::GetClassHashAt,
                GetClassHashAtParams {
                    block_id: *block_id.as_ref(),
                    contract_address: *contract_address.as_ref(),
                },
            )
            .await?
            .0)
    }

    /// Get the contract class definition in the given block at the given address
    async fn get_class_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FeltPrimitive> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetClassAt,
            GetClassAtParams {
                block_id: *block_id.as_ref(),
                contract_address: *contract_address.as_ref(),
            },
        )
        .await
    }

    /// Get the number of transactions in a block given a block id
    async fn get_block_transaction_count<B>(&self, block_id: B) -> Result<u64, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetBlockTransactionCount,
            GetBlockTransactionCountParams {
                block_id: *block_id.as_ref(),
            },
        )
        .await
    }

    /// Call a starknet function without creating a Starknet transaction
    async fn call<R, B>(&self, request: R, block_id: B) -> Result<Vec<FeltPrimitive>, ProviderError>
    where
        R: AsRef<FunctionCall> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        Ok(self
            .send_request::<_, FeltArray>(
                JsonRpcMethod::Call,
                CallParams {
                    request: request.as_ref().clone(),
                    block_id: *block_id.as_ref(),
                },
            )
            .await?
            .0)
    }

    /// Estimate the fee for a given Starknet transaction
    async fn estimate_fee<R, S, B>(
        &self,
        request: R,

        block_id: B,
    ) -> Result<Vec<FeeEstimate>, ProviderError>
    where
        R: AsRef<[BroadcastedTxn]> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::EstimateFee,
            EstimateFeeParams {
                request: request.as_ref().to_vec(),
                block_id: *block_id.as_ref(),
            },
        )
        .await
    }

    /// Estimate the L2 fee of a message sent on L1
    async fn estimate_message_fee<M, B>(
        &self,
        message: M,
        block_id: B,
    ) -> Result<FeeEstimate, ProviderError>
    where
        M: AsRef<MsgFromL1> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::EstimateMessageFee,
            EstimateMessageFeeParams {
                message: message.as_ref().clone(),
                block_id: *block_id.as_ref(),
            },
        )
        .await
    }

    /// Get the most recent accepted block number
    async fn block_number(&self) -> Result<u64, ProviderError> {
        self.send_request(JsonRpcMethod::BlockNumber, BlockNumberParams {})
            .await
    }

    /// Get the most recent accepted block hash and number
    async fn block_hash_and_number(&self) -> Result<BlockHashAndNumber, ProviderError> {
        self.send_request(
            JsonRpcMethod::BlockHashAndNumber,
            BlockHashAndNumberParams {},
        )
        .await
    }

    /// Return the currently configured Starknet chain id
    async fn chain_id(&self) -> Result<FeltPrimitive, ProviderError> {
        Ok(self
            .send_request::<_, Felt>(JsonRpcMethod::ChainId, ChainIdParams {})
            .await?
            .0)
    }

    /// Returns an object about the sync status, or false if the node is not synching
    async fn syncing(&self) -> Result<SyncingStatus, ProviderError> {
        self.send_request(JsonRpcMethod::Syncing, SyncingParams {})
            .await
    }

    /// Returns all events matching the given filter
    async fn get_events(
        &self,
        filter: EventFilterWithPageRequest,
    ) -> Result<EventsChunk, ProviderError> {
        self.send_request(JsonRpcMethod::GetEvents, GetEventsParams { filter })
            .await
    }

    /// Get the nonce associated with the given address in the given block
    async fn get_nonce<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FeltPrimitive, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FeltPrimitive> + Send + Sync,
    {
        Ok(self
            .send_request::<_, Felt>(
                JsonRpcMethod::GetNonce,
                GetNonceParams {
                    block_id: *block_id.as_ref(),
                    contract_address: *contract_address.as_ref(),
                },
            )
            .await?
            .0)
    }

    /// Submit a new transaction to be added to the chain
    async fn add_invoke_transaction<I>(
        &self,
        invoke_transaction: I,
    ) -> Result<AddInvokeTransactionResult, ProviderError>
    where
        I: AsRef<BroadcastedInvokeTxn> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::AddInvokeTransaction,
            AddInvokeTransactionParams {
                invoke_transaction: invoke_transaction.as_ref().clone(),
            },
        )
        .await
    }

    /// Submit a new transaction to be added to the chain
    async fn add_declare_transaction<D>(
        &self,
        declare_transaction: D,
    ) -> Result<ClassAndTxnHash, ProviderError>
    where
        D: AsRef<BroadcastedDeclareTxn> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::AddDeclareTransaction,
            AddDeclareTransactionParams {
                declare_transaction: declare_transaction.as_ref().clone(),
            },
        )
        .await
    }

    /// Submit a new deploy account transaction
    async fn add_deploy_account_transaction<D>(
        &self,
        deploy_account_transaction: D,
    ) -> Result<ContractAndTxnHash, ProviderError>
    where
        D: AsRef<BroadcastedDeployAccountTxn> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::AddDeployAccountTransaction,
            AddDeployAccountTransactionParams {
                deploy_account_transaction: deploy_account_transaction.as_ref().clone(),
            },
        )
        .await
    }

    /// For a given executed transaction, return the trace of its execution, including internal
    /// calls
    async fn trace_transaction<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionTrace, ProviderError>
    where
        H: AsRef<FeltPrimitive> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::TraceTransaction,
            TraceTransactionParams {
                transaction_hash: transaction_hash.as_ref().clone(),
            },
        )
        .await
    }

    /// Simulate a given sequence of transactions on the requested state, and generate the execution
    /// traces. Note that some of the transactions may revert, in which case no error is thrown, but
    /// revert details can be seen on the returned trace object. . Note that some of the
    /// transactions may revert, this will be reflected by the revert_error property in the trace.
    /// Other types of failures (e.g. unexpected error or failure in the validation phase) will
    /// result in TRANSACTION_EXECUTION_ERROR.
    async fn simulate_transactions<B, TX, S>(
        &self,
        block_id: B,
        transactions: TX,
        simulation_flags: S,
    ) -> Result<Vec<SimulateTransactionsResult>, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        TX: AsRef<[BroadcastedTxn]> + Send + Sync,
        S: AsRef<[SimulationFlag]> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::SimulateTransactions,
            SimulateTransactionsParams {
                block_id: *block_id.as_ref(),
                transactions: transactions.as_ref().to_vec(),
                simulation_flags: simulation_flags.as_ref().to_vec(),
            },
        )
        .await
    }

    /// Retrieve traces for all transactions in the given block.
    async fn trace_block_transactions<B>(
        &self,
        block_id: B,
    ) -> Result<Vec<TraceBlockTransactionsResult>, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::TraceBlockTransactions,
            TraceBlockTransactionsParams {
                block_id: *block_id.as_ref(),
            },
        )
        .await
    }
}

impl<'de> Deserialize<'de> for JsonRpcRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRequest {
            id: u64,
            method: JsonRpcMethod,
            params: serde_json::Value,
        }

        let error_mapper =
            |err| serde::de::Error::custom(format!("unable to decode params: {}", err));

        let raw_request = RawRequest::deserialize(deserializer)?;
        let request_data = match raw_request.method {
            JsonRpcMethod::SpecVersion => JsonRpcRequestData::SpecVersion(
                serde_json::from_value::<SpecVersionParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetBlockWithTxHashes => JsonRpcRequestData::GetBlockWithTxHashes(
                serde_json::from_value::<GetBlockWithTxHashesParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetBlockWithTxs => JsonRpcRequestData::GetBlockWithTxs(
                serde_json::from_value::<GetBlockWithTxsParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetStateUpdate => JsonRpcRequestData::GetStateUpdate(
                serde_json::from_value::<GetStateUpdateParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetStorageAt => JsonRpcRequestData::GetStorageAt(
                serde_json::from_value::<GetStorageAtParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetTransactionStatus => JsonRpcRequestData::GetTransactionStatus(
                serde_json::from_value::<GetTransactionStatusParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetTransactionByHash => JsonRpcRequestData::GetTransactionByHash(
                serde_json::from_value::<GetTransactionByHashParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetTransactionByBlockIdAndIndex => {
                JsonRpcRequestData::GetTransactionByBlockIdAndIndex(
                    serde_json::from_value::<GetTransactionByBlockIdAndIndexParams>(
                        raw_request.params,
                    )
                    .map_err(error_mapper)?,
                )
            }
            JsonRpcMethod::GetTransactionReceipt => JsonRpcRequestData::GetTransactionReceipt(
                serde_json::from_value::<GetTransactionReceiptParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetClass => JsonRpcRequestData::GetClass(
                serde_json::from_value::<GetClassParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetClassHashAt => JsonRpcRequestData::GetClassHashAt(
                serde_json::from_value::<GetClassHashAtParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetClassAt => JsonRpcRequestData::GetClassAt(
                serde_json::from_value::<GetClassAtParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetBlockTransactionCount => {
                JsonRpcRequestData::GetBlockTransactionCount(
                    serde_json::from_value::<GetBlockTransactionCountParams>(raw_request.params)
                        .map_err(error_mapper)?,
                )
            }
            JsonRpcMethod::Call => JsonRpcRequestData::Call(
                serde_json::from_value::<CallParams>(raw_request.params).map_err(error_mapper)?,
            ),
            JsonRpcMethod::EstimateFee => JsonRpcRequestData::EstimateFee(
                serde_json::from_value::<EstimateFeeParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::EstimateMessageFee => JsonRpcRequestData::EstimateMessageFee(
                serde_json::from_value::<EstimateMessageFeeParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::BlockNumber => JsonRpcRequestData::BlockNumber(
                serde_json::from_value::<BlockNumberParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::BlockHashAndNumber => JsonRpcRequestData::BlockHashAndNumber(
                serde_json::from_value::<BlockHashAndNumberParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::ChainId => JsonRpcRequestData::ChainId(
                serde_json::from_value::<ChainIdParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::Syncing => JsonRpcRequestData::Syncing(
                serde_json::from_value::<SyncingParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetEvents => JsonRpcRequestData::GetEvents(
                serde_json::from_value::<GetEventsParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetNonce => JsonRpcRequestData::GetNonce(
                serde_json::from_value::<GetNonceParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::AddInvokeTransaction => JsonRpcRequestData::AddInvokeTransaction(
                serde_json::from_value::<AddInvokeTransactionParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::AddDeclareTransaction => JsonRpcRequestData::AddDeclareTransaction(
                serde_json::from_value::<AddDeclareTransactionParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::AddDeployAccountTransaction => {
                JsonRpcRequestData::AddDeployAccountTransaction(
                    serde_json::from_value::<AddDeployAccountTransactionParams>(raw_request.params)
                        .map_err(error_mapper)?,
                )
            }
            JsonRpcMethod::TraceTransaction => JsonRpcRequestData::TraceTransaction(
                serde_json::from_value::<TraceTransactionParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::SimulateTransactions => JsonRpcRequestData::SimulateTransactions(
                serde_json::from_value::<SimulateTransactionsParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::TraceBlockTransactions => JsonRpcRequestData::TraceBlockTransactions(
                serde_json::from_value::<TraceBlockTransactionsParams>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
        };

        Ok(Self {
            id: raw_request.id,
            data: request_data,
        })
    }
}

impl<T> ProviderImplError for JsonRpcClientError<T>
where
    T: 'static + Error + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T> From<JsonRpcClientError<T>> for ProviderError
where
    T: 'static + Error + Send + Sync,
{
    fn from(value: JsonRpcClientError<T>) -> Self {
        Self::Other(Box::new(value))
    }
}

impl<T> From<serde_json::Error> for JsonRpcClientError<T> {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value)
    }
}

impl TryFrom<&JsonRpcError> for StarknetError {
    type Error = JsonRpcErrorConversionError;

    fn try_from(value: &JsonRpcError) -> Result<Self, Self::Error> {
        match value.code {
            1 => Ok(StarknetError::FailedToReceiveTransaction),
            20 => Ok(StarknetError::ContractNotFound),
            24 => Ok(StarknetError::BlockNotFound),
            27 => Ok(StarknetError::InvalidTransactionIndex),
            28 => Ok(StarknetError::ClassHashNotFound),
            29 => Ok(StarknetError::TransactionHashNotFound),
            31 => Ok(StarknetError::PageSizeTooBig),
            32 => Ok(StarknetError::NoBlocks),
            33 => Ok(StarknetError::InvalidContinuationToken),
            34 => Ok(StarknetError::TooManyKeysInFilter),
            40 => {
                let data = ContractErrorData::deserialize(
                    value
                        .data
                        .as_ref()
                        .ok_or(JsonRpcErrorConversionError::MissingData)?,
                )
                .map_err(|_| JsonRpcErrorConversionError::DataParsingFailure)?;
                Ok(StarknetError::ContractError(data))
            }
            41 => {
                let data = TransactionExecutionErrorData::deserialize(
                    value
                        .data
                        .as_ref()
                        .ok_or(JsonRpcErrorConversionError::MissingData)?,
                )
                .map_err(|_| JsonRpcErrorConversionError::DataParsingFailure)?;
                Ok(StarknetError::TransactionExecutionError(data))
            }
            51 => Ok(StarknetError::ClassAlreadyDeclared),
            52 => Ok(StarknetError::InvalidTransactionNonce),
            53 => Ok(StarknetError::InsufficientMaxFee),
            54 => Ok(StarknetError::InsufficientAccountBalance),
            55 => {
                let data = String::deserialize(
                    value
                        .data
                        .as_ref()
                        .ok_or(JsonRpcErrorConversionError::MissingData)?,
                )
                .map_err(|_| JsonRpcErrorConversionError::DataParsingFailure)?;
                Ok(StarknetError::ValidationFailure(data))
            }
            56 => Ok(StarknetError::CompilationFailed),
            57 => Ok(StarknetError::ContractClassSizeIsTooLarge),
            58 => Ok(StarknetError::NonAccount),
            59 => Ok(StarknetError::DuplicateTx),
            60 => Ok(StarknetError::CompiledClassHashMismatch),
            61 => Ok(StarknetError::UnsupportedTxVersion),
            62 => Ok(StarknetError::UnsupportedContractClassVersion),
            63 => {
                let data = String::deserialize(
                    value
                        .data
                        .as_ref()
                        .ok_or(JsonRpcErrorConversionError::MissingData)?,
                )
                .map_err(|_| JsonRpcErrorConversionError::DataParsingFailure)?;
                Ok(StarknetError::UnexpectedError(data))
            }
            10 => {
                let data = NoTraceAvailableErrorData::deserialize(
                    value
                        .data
                        .as_ref()
                        .ok_or(JsonRpcErrorConversionError::MissingData)?,
                )
                .map_err(|_| JsonRpcErrorConversionError::DataParsingFailure)?;
                Ok(StarknetError::NoTraceAvailable(data))
            }
            _ => Err(JsonRpcErrorConversionError::UnknownCode),
        }
    }
}

impl Error for JsonRpcError {}

impl Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            Some(data) => {
                write!(
                    f,
                    "JSON-RPC error: code={}, message=\"{}\", data={}",
                    self.code,
                    self.message,
                    serde_json::to_string(data).map_err(|_| std::fmt::Error)?
                )
            }
            None => {
                write!(
                    f,
                    "JSON-RPC error: code={}, message=\"{}\"",
                    self.code, self.message
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum StarknetError {
    /// Failed to write transaction
    FailedToReceiveTransaction,
    /// Contract not found
    ContractNotFound,
    /// Block not found
    BlockNotFound,
    /// Invalid transaction index in a block
    InvalidTransactionIndex,
    /// Class hash not found
    ClassHashNotFound,
    /// Transaction hash not found
    TransactionHashNotFound,
    /// Requested page size is too big
    PageSizeTooBig,
    /// There are no blocks
    NoBlocks,
    /// The supplied continuation token is invalid or unknown
    InvalidContinuationToken,
    /// Too many keys provided in a filter
    TooManyKeysInFilter,
    /// Contract error
    ContractError(ContractErrorData),
    /// Transaction execution error
    TransactionExecutionError(TransactionExecutionErrorData),
    /// Class already declared
    ClassAlreadyDeclared,
    /// Invalid transaction nonce
    InvalidTransactionNonce,
    /// Max fee is smaller than the minimal transaction cost (validation plus fee transfer)
    InsufficientMaxFee,
    /// Account balance is smaller than the transaction's max_fee
    InsufficientAccountBalance,
    /// Account validation failed
    ValidationFailure(String),
    /// Compilation failed
    CompilationFailed,
    /// Contract class size it too large
    ContractClassSizeIsTooLarge,
    /// Sender address in not an account contract
    NonAccount,
    /// A transaction with the same hash already exists in the mempool
    DuplicateTx,
    /// the compiled class hash did not match the one supplied in the transaction
    CompiledClassHashMismatch,
    /// the transaction version is not supported
    UnsupportedTxVersion,
    /// the contract class version is not supported
    UnsupportedContractClassVersion,
    /// An unexpected error occurred
    UnexpectedError(String),
    /// No trace available for transaction
    NoTraceAvailable(NoTraceAvailableErrorData),
}

#[cfg(feature = "std")]
impl std::error::Error for StarknetError {}

impl core::fmt::Display for StarknetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FailedToReceiveTransaction => write!(f, "FailedToReceiveTransaction"),
            Self::ContractNotFound => write!(f, "ContractNotFound"),
            Self::BlockNotFound => write!(f, "BlockNotFound"),
            Self::InvalidTransactionIndex => write!(f, "InvalidTransactionIndex"),
            Self::ClassHashNotFound => write!(f, "ClassHashNotFound"),
            Self::TransactionHashNotFound => write!(f, "TransactionHashNotFound"),
            Self::PageSizeTooBig => write!(f, "PageSizeTooBig"),
            Self::NoBlocks => write!(f, "NoBlocks"),
            Self::InvalidContinuationToken => write!(f, "InvalidContinuationToken"),
            Self::TooManyKeysInFilter => write!(f, "TooManyKeysInFilter"),
            Self::ContractError(_) => write!(f, "ContractError"),
            Self::TransactionExecutionError(_) => write!(f, "TransactionExecutionError"),
            Self::ClassAlreadyDeclared => write!(f, "ClassAlreadyDeclared"),
            Self::InvalidTransactionNonce => write!(f, "InvalidTransactionNonce"),
            Self::InsufficientMaxFee => write!(f, "InsufficientMaxFee"),
            Self::InsufficientAccountBalance => write!(f, "InsufficientAccountBalance"),
            Self::ValidationFailure(_) => write!(f, "ValidationFailure"),
            Self::CompilationFailed => write!(f, "CompilationFailed"),
            Self::ContractClassSizeIsTooLarge => write!(f, "ContractClassSizeIsTooLarge"),
            Self::NonAccount => write!(f, "NonAccount"),
            Self::DuplicateTx => write!(f, "DuplicateTx"),
            Self::CompiledClassHashMismatch => write!(f, "CompiledClassHashMismatch"),
            Self::UnsupportedTxVersion => write!(f, "UnsupportedTxVersion"),
            Self::UnsupportedContractClassVersion => write!(f, "UnsupportedContractClassVersion"),
            Self::UnexpectedError(_) => write!(f, "UnexpectedError"),
            Self::NoTraceAvailable(_) => write!(f, "NoTraceAvailable"),
        }
    }
}

impl StarknetError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::FailedToReceiveTransaction => "Failed to write transaction",
            Self::ContractNotFound => "Contract not found",
            Self::BlockNotFound => "Block not found",
            Self::InvalidTransactionIndex => "Invalid transaction index in a block",
            Self::ClassHashNotFound => "Class hash not found",
            Self::TransactionHashNotFound => "Transaction hash not found",
            Self::PageSizeTooBig => "Requested page size is too big",
            Self::NoBlocks => "There are no blocks",
            Self::InvalidContinuationToken => "The supplied continuation token is invalid or unknown",
            Self::TooManyKeysInFilter => "Too many keys provided in a filter",
            Self::ContractError(_) => "Contract error",
            Self::TransactionExecutionError(_) => "Transaction execution error",
            Self::ClassAlreadyDeclared => "Class already declared",
            Self::InvalidTransactionNonce => "Invalid transaction nonce",
            Self::InsufficientMaxFee => "Max fee is smaller than the minimal transaction cost (validation plus fee transfer)",
            Self::InsufficientAccountBalance => "Account balance is smaller than the transaction's max_fee",
            Self::ValidationFailure(_) => "Account validation failed",
            Self::CompilationFailed => "Compilation failed",
            Self::ContractClassSizeIsTooLarge => "Contract class size it too large",
            Self::NonAccount => "Sender address in not an account contract",
            Self::DuplicateTx => "A transaction with the same hash already exists in the mempool",
            Self::CompiledClassHashMismatch => "the compiled class hash did not match the one supplied in the transaction",
            Self::UnsupportedTxVersion => "the transaction version is not supported",
            Self::UnsupportedContractClassVersion => "the contract class version is not supported",
            Self::UnexpectedError(_) => "An unexpected error occurred",
            Self::NoTraceAvailable(_) => "No trace available for transaction",
        }
    }
}

/// Extra information on why trace is not available. Either it wasn't executed yet (received), or
/// the transaction failed (rejected).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct NoTraceAvailableErrorData {
    pub status: SequencerTransactionStatus,
}

/// Transaction status.
///
/// The finality status of the transaction, including the case the txn is still in the mempool or
/// failed validation during the block construction phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequencerTransactionStatus {
    #[serde(rename = "RECEIVED")]
    Received,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "ACCEPTED_ON_L2")]
    AcceptedOnL2,
    #[serde(rename = "ACCEPTED_ON_L1")]
    AcceptedOnL1,
}

/// More data about the execution failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct TransactionExecutionErrorData {
    /// The index of the first transaction failing in a sequence of given transactions
    pub transaction_index: u64,
    /// A string encoding the execution trace up to the point of failure
    pub execution_error: String,
}

/// More data about the execution failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct ContractErrorData {
    /// A string encoding the execution trace up to the point of failure
    pub revert_error: String,
}
