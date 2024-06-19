use super::{DevnetClientError, DevnetError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use starknet_crypto::FieldElement;
use std::{any::Any, error::Error};
use tracing::debug;
use utils::codegen::{
    AddDeclareTransactionRequestRef, AddInvokeTransactionRequestRef, BlockTag, CallRequestRef,
    EstimateFeeRequestRef, FeeEstimate, FunctionCall, GetBlockWithReceiptsRequestRef,
    GetBlockWithTxHashesRequestRef, GetBlockWithTxsRequestRef, GetClassRequestRef,
    GetNonceRequestRef, GetStateUpdateRequestRef, GetStorageAtRequestRef,
    GetTransactionByHashRequestRef, GetTransactionReceiptRequestRef,
    GetTransactionStatusRequestRef, SimulateTransactionsRequestRef, SimulatedTransaction,
    SimulationFlag, SimulationFlagForEstimateFee, SpecVersionRequest,
    TransactionReceiptWithBlockInfo,
};
use utils::models::{
    BlockId, BroadcastedDeclareTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction,
    DeclareTransactionResult, InvokeTransactionResult, MaybePendingBlockWithReceipts,
    MaybePendingBlockWithTxs, MaybePendingStateUpdate, Transaction, TransactionStatus,
};
use utils::transports::{
    Felt, FeltArray, JsonRpcMethod, JsonRpcResponse, MaybePendingBlockWithTxHashes,
};
use utils::{
    codegen::{
        ContractErrorData, NoTraceAvailableErrorData, StarknetError, TransactionExecutionErrorData,
    },
    models::FeeUnit,
    provider::{Provider, ProviderError, ProviderImplError},
    transports::{
        http::{HttpTransport, HttpTransportError},
        JsonRpcClientError,
    },
};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DevnetMethod {
    #[serde(rename = "get_account_balance")]
    GetAccountBalance,
    #[serde(rename = "config")]
    Config,
}
#[derive(Debug, Serialize)]
struct MintRequest {
    amount: u128,
    address: String,
}
#[derive(Debug, Serialize)]
struct IncreaseTimeRequest {
    time: u64,
}
#[derive(Debug, Serialize)]
struct SetTimeRequest {
    time: u64,
    generate_block: bool,
}
#[derive(Debug, Serialize)]
struct AbortBlocksRequest {
    starting_block_hash: String,
}

#[allow(unused)]
impl<Q> Provider for DevnetClient<Q>
where
    Q: 'static + DevnetTransport + Sync + Send,
{
    async fn spec_version(&self) -> Result<String, ProviderError> {
        self.send_request(JsonRpcMethod::SpecVersion, SpecVersionRequest)
            .await
    }

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

    async fn get_predeployed_accounts(&self) -> Result<Value, ProviderError> {
        self.send_get_request("predeployed_accounts", Option::None)
            .await
    }
    async fn get_config(&self) -> Result<Value, ProviderError> {
        self.send_get_request("config", Option::None).await
    }
    async fn get_account_balance(
        &self,
        address: FieldElement,
        unit: FeeUnit,
        block_tag: BlockTag,
    ) -> Result<Value, ProviderError> {
        let tag = match block_tag {
            BlockTag::Latest => "latest",
            BlockTag::Pending => "pending",
        };
        let params = format!("address={:#x}&unit={}&block_tag={}", address, unit, tag);

        self.send_get_request("account_balance", Some(params)).await
    }

    async fn mint(&self, address: FieldElement, mint_amount: u128) -> Result<Value, ProviderError> {
        let req = MintRequest {
            address: format!("{address:#x}"),
            amount: mint_amount,
        };

        self.send_post_request("mint", &req).await
    }

    async fn set_time(&self, time: u64, generate_block: bool) -> Result<Value, ProviderError> {
        let req = SetTimeRequest {
            time,
            generate_block,
        };
        self.send_post_request("set_time", &req).await
    }
    async fn increase_time(&self, increase_time: u64) -> Result<Value, ProviderError> {
        let req = IncreaseTimeRequest {
            time: increase_time,
        };
        self.send_post_request("increase_time", &req).await
    }

    async fn create_block(&self) -> Result<Value, ProviderError> {
        self.send_post_request("create_block", &json!({})).await
    }

    async fn abort_blocks(&self, starting_block_hash: String) -> Result<Value, ProviderError> {
        let req: AbortBlocksRequest = AbortBlocksRequest {
            starting_block_hash,
        };
        self.send_post_request("abort_blocks", &req).await
    }

    async fn get_class<B, H>(
        &self,
        block_id: B,
        class_hash: H,
    ) -> Result<utils::models::ContractClass, ProviderError>
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
}

impl<T> ProviderImplError for DevnetClientError<T>
where
    T: 'static + Error + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T> From<serde_json::Error> for DevnetClientError<T> {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value)
    }
}
/// Failures trying to parse a [JsonRpcError] into [StarknetError].
#[derive(Debug, thiserror::Error)]
pub enum DevnetConversionError {
    #[error("unknown error code")]
    UnknownCode,
    #[error("missing data field")]
    MissingData,
    #[error("unable to parse the data field")]
    DataParsingFailure,
}

impl TryFrom<&DevnetError> for StarknetError {
    type Error = DevnetConversionError;

    fn try_from(value: &DevnetError) -> Result<Self, Self::Error> {
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
                        .ok_or(DevnetConversionError::MissingData)?,
                )
                .map_err(|_| DevnetConversionError::DataParsingFailure)?;
                Ok(StarknetError::ContractError(data))
            }
            41 => {
                let data = TransactionExecutionErrorData::deserialize(
                    value
                        .data
                        .as_ref()
                        .ok_or(DevnetConversionError::MissingData)?,
                )
                .map_err(|_| DevnetConversionError::DataParsingFailure)?;
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
                        .ok_or(DevnetConversionError::MissingData)?,
                )
                .map_err(|_| DevnetConversionError::DataParsingFailure)?;
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
                        .ok_or(DevnetConversionError::MissingData)?,
                )
                .map_err(|_| DevnetConversionError::DataParsingFailure)?;
                Ok(StarknetError::UnexpectedError(data))
            }
            10 => {
                let data = NoTraceAvailableErrorData::deserialize(
                    value
                        .data
                        .as_ref()
                        .ok_or(DevnetConversionError::MissingData)?,
                )
                .map_err(|_| DevnetConversionError::DataParsingFailure)?;
                Ok(StarknetError::NoTraceAvailable(data))
            }
            _ => Err(DevnetConversionError::UnknownCode),
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait DevnetTransport {
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

    async fn send_get_request<R>(
        &self,
        method: &str,
        query: Option<String>,
    ) -> Result<R, Self::Error>
    where
        R: DeserializeOwned;
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T> {
    id: u64,
    jsonrpc: &'static str,
    method: JsonRpcMethod,
    params: T,
}

#[allow(unused)]
impl DevnetTransport for HttpTransport {
    type Error = HttpTransportError;
    async fn send_request<P, R>(
        &self,
        method: JsonRpcMethod,
        params: P,
    ) -> Result<JsonRpcResponse<R>, Self::Error>
    where
        P: Serialize + Send,
        R: DeserializeOwned,
    {
        let request_body = JsonRpcRequest {
            id: 1,
            jsonrpc: "2.0",
            method,
            params,
        };

        let request_body = serde_json::to_string(&request_body).map_err(Self::Error::Json)?;
        debug!("Sending request via JSON-RPC: {}", request_body);

        let mut request = self
            .client
            .post(self.url.clone())
            .body(request_body)
            .header("Content-Type", "application/json");
        for (name, value) in self.headers.iter() {
            request = request.header(name, value);
        }

        let response = request.send().await.map_err(Self::Error::Reqwest)?;

        let response_body = response.text().await.map_err(Self::Error::Reqwest)?;
        debug!("Response from JSON-RPC: {}", response_body);

        let parsed_response = serde_json::from_str(&response_body).map_err(Self::Error::Json)?;

        Ok(parsed_response)
    }

    async fn send_post_request<R, Q>(&self, method: &str, body: &Q) -> Result<R, Self::Error>
    where
        Q: Serialize,
        R: DeserializeOwned,
    {
        let request_body = serde_json::to_string(body).map_err(Self::Error::Json)?;

        debug!("Sending request via JSON-RPC: {}", request_body);
        let uri: String = format!("{}{}", self.url, method);

        let mut request = self
            .client
            .post(uri)
            .body(request_body)
            .header("Content-Type", "application/json");
        for (name, value) in self.headers.iter() {
            request = request.header(name, value);
        }

        let response = request.send().await.map_err(Self::Error::Reqwest)?;

        let response_body = response.text().await.map_err(Self::Error::Reqwest)?;
        debug!("Response from JSON-RPC: {}", response_body);

        let parsed_response = serde_json::from_str(&response_body).map_err(Self::Error::Json)?;

        Ok(parsed_response)
    }

    async fn send_get_request<R>(
        &self,
        method: &str,
        query: Option<String>,
    ) -> Result<R, Self::Error>
    where
        R: DeserializeOwned,
    {
        let uri = format!("{}{}?{}", self.url, method, query.unwrap_or("".into()));

        debug!("Sending GET request to URL: {}", uri);

        let mut request = self.client.get(uri);
        for (name, value) in self.headers.iter() {
            request = request.header(name, value);
        }

        let response = request.send().await.map_err(HttpTransportError::Reqwest)?;

        let response_body = response.text().await.map_err(HttpTransportError::Reqwest)?;
        debug!("Response from GET request: {}", response_body);
        let parsed_response: R =
            serde_json::from_str(&response_body).map_err(HttpTransportError::Json)?;

        Ok(parsed_response)
    }
}

#[allow(dead_code)]
impl<T> DevnetClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

#[derive(Debug, Clone)]
pub struct DevnetClient<T> {
    transport: T,
}

#[allow(unused)]
impl<T> DevnetClient<T>
where
    T: 'static + DevnetTransport + Send + Sync,
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
    async fn send_get_request<R>(
        &self,
        method: &str,
        query: Option<String>,
    ) -> Result<R, ProviderError>
    where
        R: DeserializeOwned,
    {
        Ok(self
            .transport
            .send_get_request(method, query)
            .await
            .map_err(JsonRpcClientError::TransportError)?)
    }
}
