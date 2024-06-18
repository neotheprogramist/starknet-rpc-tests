use std::{any::Any, error::Error};

use super::{DevnetClientError, DevnetError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use starknet_crypto::FieldElement;
use tracing::{debug, field::Field};
use utils::codegen::BlockTag;
use utils::{
    codegen::{
        ContractErrorData, NoTraceAvailableErrorData, StarknetError, TransactionExecutionErrorData,
    },
    models::FeeUnit,
    provider::{Config, Provider, ProviderError, ProviderImplError},
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

#[allow(unused)]
impl<Q> Provider for DevnetClient<Q>
where
    Q: 'static + DevnetTransport + Sync + Send,
{
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
    async fn spec_version(&self) -> Result<String, ProviderError> {
        todo!()
    }

    async fn get_block_with_tx_hashes<B>(
        &self,
        block_id: B,
    ) -> Result<utils::transports::MaybePendingBlockWithTxHashes, ProviderError>
    where
        B: AsRef<utils::models::BlockId> + Send + Sync,
    {
        todo!()
    }

    async fn get_block_with_txs<B>(
        &self,
        block_id: B,
    ) -> Result<utils::models::MaybePendingBlockWithTxs, ProviderError>
    where
        B: AsRef<utils::models::BlockId> + Send + Sync,
    {
        todo!()
    }

    async fn get_block_with_receipts<B>(
        &self,
        block_id: B,
    ) -> Result<utils::models::MaybePendingBlockWithReceipts, ProviderError>
    where
        B: AsRef<utils::models::BlockId> + Send + Sync,
    {
        todo!()
    }

    async fn get_state_update<B>(
        &self,
        block_id: B,
    ) -> Result<utils::models::MaybePendingStateUpdate, ProviderError>
    where
        B: AsRef<utils::models::BlockId> + Send + Sync,
    {
        todo!()
    }

    async fn get_storage_at<A, K, B>(
        &self,
        contract_address: A,
        key: K,
        block_id: B,
    ) -> Result<starknet_crypto::FieldElement, ProviderError>
    where
        A: AsRef<starknet_crypto::FieldElement> + Send + Sync,
        K: AsRef<starknet_crypto::FieldElement> + Send + Sync,
        B: AsRef<utils::models::BlockId> + Send + Sync,
    {
        todo!()
    }

    async fn get_transaction_status<H>(
        &self,
        transaction_hash: H,
    ) -> Result<utils::models::TransactionStatus, ProviderError>
    where
        H: AsRef<starknet_crypto::FieldElement> + Send + Sync,
    {
        todo!()
    }

    async fn get_transaction_by_hash<H>(
        &self,
        transaction_hash: H,
    ) -> Result<utils::models::Transaction, ProviderError>
    where
        H: AsRef<starknet_crypto::FieldElement> + Send + Sync,
    {
        todo!()
    }

    async fn get_transaction_receipt<H>(
        &self,
        transaction_hash: H,
    ) -> Result<utils::codegen::TransactionReceiptWithBlockInfo, ProviderError>
    where
        H: AsRef<starknet_crypto::FieldElement> + Send + Sync,
    {
        todo!()
    }

    async fn call<R, B>(
        &self,
        request: R,
        block_id: B,
    ) -> Result<Vec<starknet_crypto::FieldElement>, ProviderError>
    where
        R: AsRef<utils::codegen::FunctionCall> + Send + Sync,
        B: AsRef<utils::models::BlockId> + Send + Sync,
    {
        todo!()
    }

    async fn estimate_fee<R, S, B>(
        &self,
        request: R,
        simulation_flags: S,
        block_id: B,
    ) -> Result<Vec<utils::codegen::FeeEstimate>, ProviderError>
    where
        R: AsRef<[utils::models::BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[utils::codegen::SimulationFlagForEstimateFee]> + Send + Sync,
        B: AsRef<utils::models::BlockId> + Send + Sync,
    {
        todo!()
    }

    async fn get_nonce<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<starknet_crypto::FieldElement, ProviderError>
    where
        B: AsRef<utils::models::BlockId> + Send + Sync,
        A: AsRef<starknet_crypto::FieldElement> + Send + Sync,
    {
        todo!()
    }

    async fn add_invoke_transaction<I>(
        &self,
        invoke_transaction: I,
    ) -> Result<utils::models::InvokeTransactionResult, ProviderError>
    where
        I: AsRef<utils::models::BroadcastedInvokeTransaction> + Send + Sync,
    {
        todo!()
    }

    async fn add_declare_transaction<D>(
        &self,
        declare_transaction: D,
    ) -> Result<utils::models::DeclareTransactionResult, ProviderError>
    where
        D: AsRef<utils::models::BroadcastedDeclareTransaction> + Send + Sync,
    {
        todo!()
    }

    async fn simulate_transactions<B, T, S>(
        &self,
        block_id: B,
        transactions: T,
        simulation_flags: S,
    ) -> Result<Vec<utils::codegen::SimulatedTransaction>, ProviderError>
    where
        B: AsRef<utils::models::BlockId> + Send + Sync,
        T: AsRef<[utils::models::BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[utils::codegen::SimulationFlag]> + Send + Sync,
    {
        todo!();
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

impl DevnetTransport for HttpTransport {
    type Error = HttpTransportError;

    async fn send_post_request<R, Q>(&self, method: &str, body: &Q) -> Result<R, Self::Error>
    where
        Q: Serialize,
        R: DeserializeOwned,
    {
        let request_body = serde_json::to_string(body).map_err(Self::Error::Json)?;

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

impl<T> DevnetClient<T>
where
    T: 'static + DevnetTransport + Send + Sync,
{
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
