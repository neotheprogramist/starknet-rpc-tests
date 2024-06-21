use super::{DevnetClientError, DevnetError};
use num_bigint::BigUint;
use serde::Serializer;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use starknet_crypto::FieldElement;
use std::fmt::Display;
use std::num::NonZeroU128;
use std::{any::Any, error::Error};
use tracing::debug;
use url::Url;
use utils::codegen::{
    AddDeclareTransactionRequestRef, AddInvokeTransactionRequestRef, BlockNumberRequest, BlockTag,
    CallRequestRef, ChainIdRequest, EstimateFeeRequestRef, EstimateMessageFeeRequestRef,
    FeeEstimate, FunctionCall, GetBlockTransactionCountRequestRef, GetBlockWithReceiptsRequestRef,
    GetBlockWithTxHashesRequestRef, GetBlockWithTxsRequestRef, GetClassAtRequestRef,
    GetClassHashAtRequestRef, GetClassRequestRef, GetNonceRequestRef, GetStateUpdateRequestRef,
    GetStorageAtRequestRef, GetTransactionByHashRequestRef, GetTransactionReceiptRequestRef,
    GetTransactionStatusRequestRef, MsgFromL1, SimulateTransactionsRequestRef,
    SimulatedTransaction, SimulationFlag, SimulationFlagForEstimateFee, SpecVersionRequest,
    TransactionReceiptWithBlockInfo,
};
use utils::models::{
    BlockId, BroadcastedDeclareTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction,
    ContractClass, DeclareTransactionResult, InvokeTransactionResult,
    MaybePendingBlockWithReceipts, MaybePendingBlockWithTxs, MaybePendingStateUpdate, Transaction,
    TransactionStatus,
};
use utils::starknet_utils::parse_cairo_short_string;
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
struct FlushRequest {
    dry_run: bool,
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
#[derive(Debug, Serialize)]
struct LoadRequest {
    network_url: String,
    address: Option<String>,
}
#[derive(Debug, Serialize)]
struct ConsumeMessageRequest {
    l2_contract_address: String,
    l1_contract_address: String,
    payload: Vec<String>,
}
#[derive(Debug, Serialize)]
struct SendMessageRequest {
    l2_contract_address: String,
    entry_point_selector: String,
    l1_contract_address: String,
    payload: Vec<String>,
    paid_fee_on_l1: String,
    nonce: String,
}
pub type Balance = BigUint;
pub const MAINNET: FieldElement = FieldElement::from_mont([
    17696389056366564951,
    18446744073709551615,
    18446744073709551615,
    502562008147966918,
]);

pub const TESTNET: FieldElement = FieldElement::from_mont([
    3753493103916128178,
    18446744073709548950,
    18446744073709551615,
    398700013197595345,
]);

pub const TESTNET2: FieldElement = FieldElement::from_mont([
    1663542769632127759,
    18446744073708869172,
    18446744073709551615,
    33650220878420990,
]);

pub const SEPOLIA: FieldElement = FieldElement::from_mont([
    1555806712078248243,
    18446744073708869172,
    18446744073709551615,
    507980251676163170,
]);
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
#[clap(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChainId {
    Mainnet,
    Testnet,
}

impl ChainId {
    pub fn goerli_legacy_id() -> FieldElement {
        TESTNET.into()
    }

    pub fn to_felt(&self) -> FieldElement {
        FieldElement::from(self).into()
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let felt = FieldElement::from(self);
        let str = parse_cairo_short_string(&felt).map_err(|_| std::fmt::Error)?;
        f.write_str(&str)
    }
}

impl From<ChainId> for FieldElement {
    fn from(value: ChainId) -> Self {
        match value {
            ChainId::Mainnet => MAINNET,
            ChainId::Testnet => SEPOLIA,
        }
    }
}

impl From<&ChainId> for FieldElement {
    fn from(value: &ChainId) -> Self {
        match value {
            ChainId::Mainnet => MAINNET,
            ChainId::Testnet => SEPOLIA,
        }
    }
}

pub fn serialize_initial_balance<S>(balance: &Balance, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&balance.to_str_radix(10))
}

pub fn serialize_chain_id<S>(chain_id: &ChainId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{chain_id}"))
}
#[derive(Clone, Debug, Serialize)]
pub struct StarknetConfig {
    pub seed: u32,
    pub total_accounts: u8,
    #[serde(skip_serializing)]
    pub account_contract_class: ContractClass,
    pub account_contract_class_hash: FieldElement,
    #[serde(serialize_with = "serialize_initial_balance")]
    pub predeployed_accounts_initial_balance: Balance,
    pub start_time: Option<u64>,
    pub gas_price_wei: NonZeroU128,
    pub gas_price_strk: NonZeroU128,
    pub data_gas_price_wei: NonZeroU128,
    pub data_gas_price_strk: NonZeroU128,
    #[serde(serialize_with = "serialize_chain_id")]
    pub chain_id: ChainId,
    pub dump_on: Option<DumpOn>,
    pub dump_path: Option<String>,
    pub blocks_on_demand: bool,
    pub lite_mode: bool,
    /// on initialization, re-execute loaded txs (if any)
    #[serde(skip_serializing)]
    pub re_execute_on_init: bool,
    pub state_archive: StateArchiveCapacity,
    pub fork_config: ForkConfig,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DumpOn {
    Exit,
    Block,
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum StateArchiveCapacity {
    #[default]
    None,
    Full,
}

pub fn serialize_config_url<S>(url: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match url {
        Some(url) => serializer.serialize_str(url.as_ref()),
        None => serializer.serialize_none(),
    }
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct ForkConfig {
    #[serde(serialize_with = "serialize_config_url")]
    pub url: Option<Url>,
    pub block_number: Option<u64>,
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
    async fn chain_id(&self) -> Result<FieldElement, ProviderError> {
        Ok(self
            .send_request::<_, Felt>(JsonRpcMethod::ChainId, ChainIdRequest)
            .await?
            .0)
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
    async fn get_config(&self) -> Result<StarknetConfig, ProviderError> {
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

    async fn mint(&self, address: String, mint_amount: u128) -> Result<Value, ProviderError> {
        let req = MintRequest {
            address,
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
    async fn load(
        &self,
        network_url: String,
        address: Option<String>,
    ) -> Result<Value, ProviderError> {
        let req: LoadRequest = LoadRequest {
            network_url,
            address,
        };
        self.send_post_request("postman/load_l1_messaging_contract", &req)
            .await
    }

    async fn flush(&self, dry_run: bool) -> Result<Value, ProviderError> {
        let req: FlushRequest = FlushRequest { dry_run };
        self.send_post_request("postman/flush", &req).await
    }

    async fn consume_message_from_l2(
        &self,
        l2_contract_address: String,
        l1_contract_address: String,
        payload: Vec<String>,
    ) -> Result<Value, ProviderError> {
        let req: ConsumeMessageRequest = ConsumeMessageRequest {
            l2_contract_address,
            l1_contract_address,
            payload,
        };

        self.send_post_request("postman/consume_message_from_l2", &req)
            .await
    }

    async fn send_message_to_l2(
        &self,
        l2_contract_address: String,
        entry_point_selector: String,
        l1_contract_address: String,
        payload: Vec<String>,
        paid_fee_on_l1: String,
        nonce: String,
    ) -> Result<Value, ProviderError> {
        let req: SendMessageRequest = SendMessageRequest {
            l2_contract_address,
            entry_point_selector,
            l1_contract_address,
            payload,
            paid_fee_on_l1,
            nonce,
        };
        self.send_post_request("postman/send_message_to_l2", &req)
            .await
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

    /// Get the most recent accepted block number
    async fn block_number(&self) -> Result<u64, ProviderError> {
        self.send_request(JsonRpcMethod::BlockNumber, BlockNumberRequest)
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
