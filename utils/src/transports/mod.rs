use auto_impl::auto_impl;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use starknet_crypto::FieldElement;
use std::{any::Any, error::Error, fmt::Display};
pub mod http;
use crate::{
    codegen::{
        AddDeclareTransactionRequest, AddDeclareTransactionRequestRef,
        AddDeployAccountTransactionRequest, AddDeployAccountTransactionRequestRef,
        AddInvokeTransactionRequest, AddInvokeTransactionRequestRef, BlockNumberRequest, BlockTag,
        BlockWithTxHashes, CallRequest, CallRequestRef, ChainIdRequest, ContractErrorData,
        EstimateFeeRequest, EstimateFeeRequestRef, EstimateMessageFeeRequest,
        EstimateMessageFeeRequestRef, FeeEstimate, FunctionCall, FunctionInvocation,
        GetBlockTransactionCountRequest, GetBlockTransactionCountRequestRef,
        GetBlockWithReceiptsRequest, GetBlockWithReceiptsRequestRef, GetBlockWithTxHashesRequest,
        GetBlockWithTxHashesRequestRef, GetBlockWithTxsRequest, GetBlockWithTxsRequestRef,
        GetClassAtRequest, GetClassAtRequestRef, GetClassHashAtRequest, GetClassHashAtRequestRef,
        GetClassRequest, GetClassRequestRef, GetNonceRequest, GetNonceRequestRef,
        GetStateUpdateRequest, GetStateUpdateRequestRef, GetStorageAtRequest,
        GetStorageAtRequestRef, GetTransactionByHashRequest, GetTransactionByHashRequestRef,
        GetTransactionReceiptRequest, GetTransactionReceiptRequestRef, GetTransactionStatusRequest,
        GetTransactionStatusRequestRef, MsgFromL1, NoTraceAvailableErrorData,
        PendingBlockWithTxHashes, ResourcePrice, RevertedInvocation, SimulateTransactionsRequest,
        SimulateTransactionsRequestRef, SimulatedTransaction, SimulationFlag,
        SimulationFlagForEstimateFee, SpecVersionRequest, StarknetError,
        TransactionExecutionErrorData, TransactionReceiptWithBlockInfo,
    },
    models::{
        BlockId, BroadcastedDeclareTransaction, BroadcastedDeployAccountTransaction,
        BroadcastedInvokeTransaction, BroadcastedTransaction, ContractClass,
        DeclareTransactionResult, FeeUnit, InvokeTransactionResult, MaybePendingBlockWithReceipts,
        MaybePendingBlockWithTxs, MaybePendingStateUpdate, Transaction, TransactionStatus,
    },
    provider::{Provider, ProviderError, ProviderImplError},
    unsigned_field_element::UfeHex,
};
use serde_with::serde_as;
use std::fmt::Debug;

#[allow(async_fn_in_trait)]
#[auto_impl(&, Box, Arc)]
pub trait JsonRpcTransport {
    type Error: Error + Send + Sync;

    async fn send_request<P, R>(
        &self,
        method: JsonRpcMethod,
        params: P,
    ) -> Result<JsonRpcResponse<R>, Self::Error>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned;

    async fn send_post_request<R, Q>(&self, method: &str, body: &Q) -> Result<R, Self::Error>
    where
        Q: Serialize,
        R: DeserializeOwned;

    async fn send_get_request<R>(&self, method: &str) -> Result<R, Self::Error>
    where
        R: DeserializeOwned;
}

#[derive(Debug, Clone)]
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
    #[serde(rename = "starknet_getBlockWithReceipts")]
    GetBlockWithReceipts,
    #[serde(rename = "starknet_getStateUpdate")]
    GetStateUpdate,
    #[serde(rename = "starknet_getStorageAt")]
    GetStorageAt,
    #[serde(rename = "starknet_getTransactionStatus")]
    GetTransactionStatus,
    #[serde(rename = "starknet_getTransactionByHash")]
    GetTransactionByHash,
    // #[serde(rename = "starknet_getTransactionByBlockIdAndIndex")]
    // GetTransactionByBlockIdAndIndex,
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
    // #[serde(rename = "starknet_blockHashAndNumber")]
    // BlockHashAndNumber,
    #[serde(rename = "starknet_chainId")]
    ChainId,
    // #[serde(rename = "starknet_syncing")]
    // Syncing,
    // #[serde(rename = "starknet_getEvents")]
    // GetEvents,
    #[serde(rename = "starknet_getNonce")]
    GetNonce,
    #[serde(rename = "starknet_addInvokeTransaction")]
    AddInvokeTransaction,
    #[serde(rename = "starknet_addDeclareTransaction")]
    AddDeclareTransaction,
    #[serde(rename = "starknet_addDeployAccountTransaction")]
    AddDeployAccountTransaction,
    // #[serde(rename = "starknet_traceTransaction")]
    // TraceTransaction,
    #[serde(rename = "starknet_simulateTransactions")]
    SimulateTransactions,
    // #[serde(rename = "starknet_traceBlockTransactions")]
    // TraceBlockTransactions,
}

#[derive(Debug, Clone)]
pub struct JsonRpcRequest {
    pub id: u64,
    pub data: JsonRpcRequestData,
}

#[derive(Debug, Clone)]
pub enum JsonRpcRequestData {
    AddInvokeTransaction(AddInvokeTransactionRequest),
    AddDeclareTransaction(AddDeclareTransactionRequest),
    AddDeployAccountTransaction(AddDeployAccountTransactionRequest),
    Call(CallRequest),
    GetNonce(GetNonceRequest),
    SpecVersion(SpecVersionRequest),
    GetBlockWithTxHashes(GetBlockWithTxHashesRequest),
    GetBlockWithTxs(GetBlockWithTxsRequest),
    GetBlockWithReceipts(GetBlockWithReceiptsRequest),
    GetStateUpdate(GetStateUpdateRequest),
    GetStorageAt(GetStorageAtRequest),
    GetTransactionStatus(GetTransactionStatusRequest),
    GetTransactionByHash(GetTransactionByHashRequest),
    // GetTransactionByBlockIdAndIndex(GetTransactionByBlockIdAndIndexRequest),
    GetTransactionReceipt(GetTransactionReceiptRequest),
    GetClass(GetClassRequest),
    GetClassHashAt(GetClassHashAtRequest),
    GetClassAt(GetClassAtRequest),
    GetBlockTransactionCount(GetBlockTransactionCountRequest),
    EstimateFee(EstimateFeeRequest),
    EstimateMessageFee(EstimateMessageFeeRequest),
    BlockNumber(BlockNumberRequest),
    // BlockHashAndNumber(BlockHashAndNumberRequest),
    ChainId(ChainIdRequest),
    // Syncing(SyncingRequest),
    // GetEvents(GetEventsRequest),
    //
    // TraceTransaction(TraceTransactionRequest),
    SimulateTransactions(SimulateTransactionsRequest),
    // TraceBlockTransactions(TraceBlockTransactionsRequest),
}
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
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

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Felt(#[serde_as(as = "UfeHex")] pub FieldElement);

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct FeltArray(#[serde_as(as = "Vec<UfeHex>")] pub Vec<FieldElement>);

#[allow(dead_code)]
impl<T> JsonRpcClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

#[allow(unused)]
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

    async fn send_post_request<R, Q>(&self, method: &str, body: &Q) -> Result<R, ProviderError>
    where
        Q: Serialize,
        R: DeserializeOwned,
    {
        Ok(self
            .transport
            .send_post_request(method, body)
            .await
            .map_err(JsonRpcClientError::TransportError)?)
    }
    async fn send_get_request<R>(&self, method: &str) -> Result<R, ProviderError>
    where
        R: DeserializeOwned,
    {
        Ok(self
            .transport
            .send_get_request(method)
            .await
            .map_err(JsonRpcClientError::TransportError)?)
    }
}

#[allow(unused)]
impl<T> Provider for JsonRpcClient<T>
where
    T: 'static + JsonRpcTransport + Sync + Send,
{
    /// Returns the version of the Starknet JSON-RPC specification being used
    async fn spec_version(&self) -> Result<String, ProviderError> {
        self.send_request(JsonRpcMethod::SpecVersion, SpecVersionRequest)
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
            GetBlockWithTxHashesRequestRef {
                block_id: block_id.as_ref(),
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
            GetBlockWithTxsRequestRef {
                block_id: block_id.as_ref(),
            },
        )
        .await
    }

    /// Get block information with full transactions and receipts given the block id
    async fn get_block_with_receipts<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithReceipts, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetBlockWithReceipts,
            GetBlockWithReceiptsRequestRef {
                block_id: block_id.as_ref(),
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
            GetStateUpdateRequestRef {
                block_id: block_id.as_ref(),
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
    ) -> Result<FieldElement, ProviderError>
    where
        A: AsRef<FieldElement> + Send + Sync,
        K: AsRef<FieldElement> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        Ok(self
            .send_request::<_, Felt>(
                JsonRpcMethod::GetStorageAt,
                GetStorageAtRequestRef {
                    contract_address: contract_address.as_ref(),
                    key: key.as_ref(),
                    block_id: block_id.as_ref(),
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
    ) -> Result<TransactionStatus, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetTransactionStatus,
            GetTransactionStatusRequestRef {
                transaction_hash: transaction_hash.as_ref(),
            },
        )
        .await
    }

    /// Get the details and status of a submitted transaction
    async fn get_transaction_by_hash<H>(
        &self,
        transaction_hash: H,
    ) -> Result<Transaction, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetTransactionByHash,
            GetTransactionByHashRequestRef {
                transaction_hash: transaction_hash.as_ref(),
            },
        )
        .await
    }

    // /// Get the details of a transaction by a given block id and index
    // async fn get_transaction_by_block_id_and_index<B>(
    //     &self,
    //     block_id: B,
    //     index: u64,
    // ) -> Result<Transaction, ProviderError>
    // where
    //     B: AsRef<BlockId> + Send + Sync,
    // {
    //     self.send_request(
    //         JsonRpcMethod::GetTransactionByBlockIdAndIndex,
    //         GetTransactionByBlockIdAndIndexRequestRef {
    //             block_id: block_id.as_ref(),
    //             index: &index,
    //         },
    //     )
    //     .await
    // }

    /// Get the details of a transaction by a given block number and index
    async fn get_transaction_receipt<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionReceiptWithBlockInfo, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetTransactionReceipt,
            GetTransactionReceiptRequestRef {
                transaction_hash: transaction_hash.as_ref(),
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
        H: AsRef<FieldElement> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetClass,
            GetClassRequestRef {
                block_id: block_id.as_ref(),
                class_hash: class_hash.as_ref(),
            },
        )
        .await
    }

    /// Get the contract class hash in the given block for the contract deployed at the given address
    async fn get_class_hash_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FieldElement, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync,
    {
        Ok(self
            .send_request::<_, Felt>(
                JsonRpcMethod::GetClassHashAt,
                GetClassHashAtRequestRef {
                    block_id: block_id.as_ref(),
                    contract_address: contract_address.as_ref(),
                },
            )
            .await?
            .0)
    }

    async fn get_class_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::GetClassAt,
            GetClassAtRequestRef {
                block_id: block_id.as_ref(),
                contract_address: contract_address.as_ref(),
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
            GetBlockTransactionCountRequestRef {
                block_id: block_id.as_ref(),
            },
        )
        .await
    }

    /// Call a starknet function without creating a Starknet transaction
    async fn call<R, B>(&self, request: R, block_id: B) -> Result<Vec<FieldElement>, ProviderError>
    where
        R: AsRef<FunctionCall> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        Ok(self
            .send_request::<_, FeltArray>(
                JsonRpcMethod::Call,
                CallRequestRef {
                    request: request.as_ref(),
                    block_id: block_id.as_ref(),
                },
            )
            .await?
            .0)
    }

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
        B: AsRef<BlockId> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::EstimateFee,
            EstimateFeeRequestRef {
                request: request.as_ref(),
                simulation_flags: simulation_flags.as_ref(),
                block_id: block_id.as_ref(),
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
            EstimateMessageFeeRequestRef {
                message: message.as_ref(),
                block_id: block_id.as_ref(),
            },
        )
        .await
    }

    /// Get the most recent accepted block number
    async fn block_number(&self) -> Result<u64, ProviderError> {
        self.send_request(JsonRpcMethod::BlockNumber, BlockNumberRequest)
            .await
    }

    // /// Get the most recent accepted block hash and number
    // async fn block_hash_and_number(&self) -> Result<BlockHashAndNumber, ProviderError> {
    //     self.send_request(JsonRpcMethod::BlockHashAndNumber, BlockHashAndNumberRequest)
    //         .await
    // }

    /// Return the currently configured Starknet chain id
    async fn chain_id(&self) -> Result<FieldElement, ProviderError> {
        Ok(self
            .send_request::<_, Felt>(JsonRpcMethod::ChainId, ChainIdRequest)
            .await?
            .0)
    }

    // /// Returns an object about the sync status, or false if the node is not synching
    // async fn syncing(&self) -> Result<SyncStatusType, ProviderError> {
    //     self.send_request(JsonRpcMethod::Syncing, SyncingRequest)
    //         .await
    // }

    // /// Returns all events matching the given filter
    // async fn get_events(
    //     &self,
    //     filter: EventFilter,
    //     continuation_token: Option<String>,
    //     chunk_size: u64,
    // ) -> Result<EventsPage, ProviderError> {
    //     self.send_request(
    //         JsonRpcMethod::GetEvents,
    //         GetEventsRequestRef {
    //             filter: &EventFilterWithPage {
    //                 event_filter: filter,
    //                 result_page_request: ResultPageRequest {
    //                     continuation_token,
    //                     chunk_size,
    //                 },
    //             },
    //         },
    //     )
    //     .await
    // }

    /// Get the nonce associated with the given address in the given block
    async fn get_nonce<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FieldElement, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync,
    {
        Ok(self
            .send_request::<_, Felt>(
                JsonRpcMethod::GetNonce,
                GetNonceRequestRef {
                    block_id: block_id.as_ref(),
                    contract_address: contract_address.as_ref(),
                },
            )
            .await?
            .0)
    }

    /// Submit a new transaction to be added to the chain
    async fn add_invoke_transaction<I>(
        &self,
        invoke_transaction: I,
    ) -> Result<InvokeTransactionResult, ProviderError>
    where
        I: AsRef<BroadcastedInvokeTransaction> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::AddInvokeTransaction,
            AddInvokeTransactionRequestRef {
                invoke_transaction: invoke_transaction.as_ref(),
            },
        )
        .await
    }

    /// Submit a new transaction to be added to the chain
    async fn add_declare_transaction<D>(
        &self,
        declare_transaction: D,
    ) -> Result<DeclareTransactionResult, ProviderError>
    where
        D: AsRef<BroadcastedDeclareTransaction> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::AddDeclareTransaction,
            AddDeclareTransactionRequestRef {
                declare_transaction: declare_transaction.as_ref(),
            },
        )
        .await
    }

    // /// Submit a new deploy account transaction
    // async fn add_deploy_account_transaction<D>(
    //     &self,
    //     deploy_account_transaction: D,
    // ) -> Result<DeployAccountTransactionResult, ProviderError>
    // where
    //     D: AsRef<BroadcastedDeployAccountTransaction> + Send + Sync,
    // {
    //     self.send_request(
    //         JsonRpcMethod::AddDeployAccountTransaction,
    //         AddDeployAccountTransactionRequestRef {
    //             deploy_account_transaction: deploy_account_transaction.as_ref(),
    //         },
    //     )
    //     .await
    // }

    // /// For a given executed transaction, return the trace of its execution, including internal
    // /// calls
    // async fn trace_transaction<H>(
    //     &self,
    //     transaction_hash: H,
    // ) -> Result<TransactionTrace, ProviderError>
    // where
    //     H: AsRef<FieldElement> + Send + Sync,
    // {
    //     self.send_request(
    //         JsonRpcMethod::TraceTransaction,
    //         TraceTransactionRequestRef {
    //             transaction_hash: transaction_hash.as_ref(),
    //         },
    //     )
    //     .await
    // }

    //  Simulate a given sequence of transactions on the requested state, and generate the execution
    //  traces. Note that some of the transactions may revert, in which case no error is thrown, but
    //  revert details can be seen on the returned trace object. . Note that some of the
    // transactions may revert, this will be reflected by the revert_error property in the trace.
    // Other types of failures (e.g. unexpected error or failure in the validation phase) will
    // result in TRANSACTION_EXECUTION_ERROR.

    async fn simulate_transactions<B, TX, S>(
        &self,
        block_id: B,
        transactions: TX,
        simulation_flags: S,
    ) -> Result<Vec<SimulatedTransaction>, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        TX: AsRef<[BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[SimulationFlag]> + Send + Sync,
    {
        self.send_request(
            JsonRpcMethod::SimulateTransactions,
            SimulateTransactionsRequestRef {
                block_id: block_id.as_ref(),
                transactions: transactions.as_ref(),
                simulation_flags: simulation_flags.as_ref(),
            },
        )
        .await
    }

    async fn get_config(&self) -> Result<Value, ProviderError> {
        let result = self.send_get_request::<Value>("/config").await?;
        Ok(result)
    }

    async fn get_account_balance(
        &self,
        address: FieldElement,
        unit: FeeUnit,
        block_tag: BlockTag,
    ) -> Result<Value, ProviderError> {
        todo!()
    }

    async fn mint(&self, address: FieldElement, mint_amount: u128) -> Result<Value, ProviderError> {
        todo!()
    }

    async fn get_predeployed_accounts(&self) -> Result<Value, ProviderError> {
        todo!()
    }
    async fn set_time(&self, time: u64, generate_block: bool) -> Result<Value, ProviderError> {
        todo!()
    }
    async fn increase_time(&self, increase_time: u64) -> Result<Value, ProviderError> {
        todo!()
    }

    async fn create_block(&self) -> Result<Value, ProviderError> {
        todo!()
    }
    async fn abort_blocks(&self, starting_block_hash: String) -> Result<Value, ProviderError> {
        todo!()
    }

    // /// Retrieve traces for all transactions in the given block.
    // async fn trace_block_transactions<B>(
    //     &self,
    //     block_id: B,
    // ) -> Result<Vec<TransactionTraceWithHash>, ProviderError>
    // where
    //     B: AsRef<BlockId> + Send + Sync,
    // {
    //     self.send_request(
    //         JsonRpcMethod::TraceBlockTransactions,
    //         TraceBlockTransactionsRequestRef {
    //             block_id: block_id.as_ref(),
    //         },
    //     )
    //     .await
    // }
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
                serde_json::from_value::<SpecVersionRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetBlockWithTxHashes => JsonRpcRequestData::GetBlockWithTxHashes(
                serde_json::from_value::<GetBlockWithTxHashesRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetBlockWithTxs => JsonRpcRequestData::GetBlockWithTxs(
                serde_json::from_value::<GetBlockWithTxsRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetBlockWithReceipts => JsonRpcRequestData::GetBlockWithReceipts(
                serde_json::from_value::<GetBlockWithReceiptsRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetStateUpdate => JsonRpcRequestData::GetStateUpdate(
                serde_json::from_value::<GetStateUpdateRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetStorageAt => JsonRpcRequestData::GetStorageAt(
                serde_json::from_value::<GetStorageAtRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetTransactionStatus => JsonRpcRequestData::GetTransactionStatus(
                serde_json::from_value::<GetTransactionStatusRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetTransactionByHash => JsonRpcRequestData::GetTransactionByHash(
                serde_json::from_value::<GetTransactionByHashRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            // JsonRpcMethod::GetTransactionByBlockIdAndIndex => {
            //     JsonRpcRequestData::GetTransactionByBlockIdAndIndex(
            //         serde_json::from_value::<GetTransactionByBlockIdAndIndexRequest>(
            //             raw_request.params,
            //         )
            //         .map_err(error_mapper)?,
            //     )
            // }
            JsonRpcMethod::GetTransactionReceipt => JsonRpcRequestData::GetTransactionReceipt(
                serde_json::from_value::<GetTransactionReceiptRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetClass => JsonRpcRequestData::GetClass(
                serde_json::from_value::<GetClassRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetClassHashAt => JsonRpcRequestData::GetClassHashAt(
                serde_json::from_value::<GetClassHashAtRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetClassAt => JsonRpcRequestData::GetClassAt(
                serde_json::from_value::<GetClassAtRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::GetBlockTransactionCount => {
                JsonRpcRequestData::GetBlockTransactionCount(
                    serde_json::from_value::<GetBlockTransactionCountRequest>(raw_request.params)
                        .map_err(error_mapper)?,
                )
            }
            JsonRpcMethod::Call => JsonRpcRequestData::Call(
                serde_json::from_value::<CallRequest>(raw_request.params).map_err(error_mapper)?,
            ),
            JsonRpcMethod::EstimateFee => JsonRpcRequestData::EstimateFee(
                serde_json::from_value::<EstimateFeeRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::EstimateMessageFee => JsonRpcRequestData::EstimateMessageFee(
                serde_json::from_value::<EstimateMessageFeeRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::BlockNumber => JsonRpcRequestData::BlockNumber(
                serde_json::from_value::<BlockNumberRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            // JsonRpcMethod::BlockHashAndNumber => JsonRpcRequestData::BlockHashAndNumber(
            //     serde_json::from_value::<BlockHashAndNumberRequest>(raw_request.params)
            //         .map_err(error_mapper)?,
            // ),
            JsonRpcMethod::ChainId => JsonRpcRequestData::ChainId(
                serde_json::from_value::<ChainIdRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            // JsonRpcMethod::Syncing => JsonRpcRequestData::Syncing(
            //     serde_json::from_value::<SyncingRequest>(raw_request.params)
            //         .map_err(error_mapper)?,
            // ),
            // JsonRpcMethod::GetEvents => JsonRpcRequestData::GetEvents(
            //     serde_json::from_value::<GetEventsRequest>(raw_request.params)
            //         .map_err(error_mapper)?,
            // ),
            JsonRpcMethod::GetNonce => JsonRpcRequestData::GetNonce(
                serde_json::from_value::<GetNonceRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::AddInvokeTransaction => JsonRpcRequestData::AddInvokeTransaction(
                serde_json::from_value::<AddInvokeTransactionRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::AddDeclareTransaction => JsonRpcRequestData::AddDeclareTransaction(
                serde_json::from_value::<AddDeclareTransactionRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            JsonRpcMethod::AddDeployAccountTransaction => {
                JsonRpcRequestData::AddDeployAccountTransaction(
                    serde_json::from_value::<AddDeployAccountTransactionRequest>(
                        raw_request.params,
                    )
                    .map_err(error_mapper)?,
                )
            } // JsonRpcMethod::TraceTransaction => JsonRpcRequestData::TraceTransaction(
            //     serde_json::from_value::<TraceTransactionRequest>(raw_request.params)
            //         .map_err(error_mapper)?,
            // ),
            JsonRpcMethod::SimulateTransactions => JsonRpcRequestData::SimulateTransactions(
                serde_json::from_value::<SimulateTransactionsRequest>(raw_request.params)
                    .map_err(error_mapper)?,
            ),
            // JsonRpcMethod::TraceBlockTransactions => JsonRpcRequestData::TraceBlockTransactions(
            //     serde_json::from_value::<TraceBlockTransactionsRequest>(raw_request.params)
            //         .map_err(error_mapper)?,
            // ),
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
impl Serialize for AddDeployAccountTransactionRequest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(transparent)]
        struct Field0<'a> {
            pub deploy_account_transaction: &'a BroadcastedDeployAccountTransaction,
        }

        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(None)?;

        seq.serialize_element(&Field0 {
            deploy_account_transaction: &self.deploy_account_transaction,
        })?;

        seq.end()
    }
}

impl<'a> Serialize for AddDeployAccountTransactionRequestRef<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(transparent)]
        struct Field0<'a> {
            pub deploy_account_transaction: &'a BroadcastedDeployAccountTransaction,
        }

        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(None)?;

        seq.serialize_element(&Field0 {
            deploy_account_transaction: self.deploy_account_transaction,
        })?;

        seq.end()
    }
}

impl<'de> Deserialize<'de> for AddDeployAccountTransactionRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[serde_as]
        #[derive(Deserialize)]
        struct AsObject {
            pub deploy_account_transaction: BroadcastedDeployAccountTransaction,
        }

        #[derive(Deserialize)]
        #[serde(transparent)]
        struct Field0 {
            pub deploy_account_transaction: BroadcastedDeployAccountTransaction,
        }

        let temp = serde_json::Value::deserialize(deserializer)?;

        if let Ok(mut elements) = Vec::<serde_json::Value>::deserialize(&temp) {
            let field0 = serde_json::from_value::<Field0>(
                elements
                    .pop()
                    .ok_or_else(|| serde::de::Error::custom("invalid sequence length"))?,
            )
            .map_err(|err| serde::de::Error::custom(format!("failed to parse element: {}", err)))?;

            Ok(Self {
                deploy_account_transaction: field0.deploy_account_transaction,
            })
        } else if let Ok(object) = AsObject::deserialize(&temp) {
            Ok(Self {
                deploy_account_transaction: object.deploy_account_transaction,
            })
        } else {
            Err(serde::de::Error::custom("invalid sequence length"))
        }
    }
}

impl Serialize for AddInvokeTransactionRequest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(transparent)]
        struct Field0<'a> {
            pub invoke_transaction: &'a BroadcastedInvokeTransaction,
        }

        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(None)?;

        seq.serialize_element(&Field0 {
            invoke_transaction: &self.invoke_transaction,
        })?;

        seq.end()
    }
}

impl<'a> Serialize for AddInvokeTransactionRequestRef<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(transparent)]
        struct Field0<'a> {
            pub invoke_transaction: &'a BroadcastedInvokeTransaction,
        }

        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(None)?;

        seq.serialize_element(&Field0 {
            invoke_transaction: self.invoke_transaction,
        })?;

        seq.end()
    }
}

impl<'de> Deserialize<'de> for AddInvokeTransactionRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[serde_as]
        #[derive(Deserialize)]
        struct AsObject {
            pub invoke_transaction: BroadcastedInvokeTransaction,
        }

        #[derive(Deserialize)]
        #[serde(transparent)]
        struct Field0 {
            pub invoke_transaction: BroadcastedInvokeTransaction,
        }

        let temp = serde_json::Value::deserialize(deserializer)?;

        if let Ok(mut elements) = Vec::<serde_json::Value>::deserialize(&temp) {
            let field0 = serde_json::from_value::<Field0>(
                elements
                    .pop()
                    .ok_or_else(|| serde::de::Error::custom("invalid sequence length"))?,
            )
            .map_err(|err| serde::de::Error::custom(format!("failed to parse element: {}", err)))?;

            Ok(Self {
                invoke_transaction: field0.invoke_transaction,
            })
        } else if let Ok(object) = AsObject::deserialize(&temp) {
            Ok(Self {
                invoke_transaction: object.invoke_transaction,
            })
        } else {
            Err(serde::de::Error::custom("invalid sequence length"))
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ExecuteInvocation {
    Success(FunctionInvocation),
    Reverted(RevertedInvocation),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybePendingBlockWithTxHashes {
    Block(BlockWithTxHashes),
    PendingBlock(PendingBlockWithTxHashes),
}

impl MaybePendingBlockWithTxHashes {
    pub fn transactions(&self) -> &[FieldElement] {
        match self {
            MaybePendingBlockWithTxHashes::Block(block) => &block.transactions,
            MaybePendingBlockWithTxHashes::PendingBlock(block) => &block.transactions,
        }
    }

    pub fn l1_gas_price(&self) -> &ResourcePrice {
        match self {
            MaybePendingBlockWithTxHashes::Block(block) => &block.l1_gas_price,
            MaybePendingBlockWithTxHashes::PendingBlock(block) => &block.l1_gas_price,
        }
    }
}
