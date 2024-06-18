use std::{any::Any, error::Error};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::debug;
use utils::{
    codegen::{
        ContractErrorData, NoTraceAvailableErrorData, StarknetError, TransactionExecutionErrorData,
    },
    provider::ProviderError,
    transports::http::{HttpTransport, HttpTransportError},
};

use super::{
    devnet_provider::{DevnetProvider, DevnetProviderError, ProviderImplError},
    Config, DevnetClientError, DevnetError,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DevnetMethod {
    #[serde(rename = "get_account_balance")]
    GetAccountBalance,
    #[serde(rename = "config")]
    Config,
}

#[derive(Debug, Deserialize)]
pub struct DevnetResponse<T> {
    result: T,
}

#[allow(async_fn_in_trait)]
pub trait DevnetTransport {
    type Error: Error + Send + Sync;

    async fn send_post_request<P, R>(
        &self,
        method: DevnetMethod,
        params: P,
    ) -> Result<DevnetResponse<R>, Self::Error>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned;

    async fn send_get_request<P, R>(&self, method: &str) -> Result<DevnetResponse<R>, Self::Error>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned;
}

#[derive(Debug, Serialize)]
struct DevnetRequest<T> {
    id: u64,
    jsonrpc: &'static str,
    method: DevnetMethod,
    params: T,
}

impl DevnetTransport for HttpTransport {
    type Error = HttpTransportError;

    async fn send_post_request<P, R>(
        &self,
        method: DevnetMethod,
        params: P,
    ) -> Result<DevnetResponse<R>, Self::Error>
    where
        P: Serialize + Send,
        R: DeserializeOwned,
    {
        let request_body = DevnetRequest {
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

    async fn send_get_request<P, R>(&self, method: &str) -> Result<DevnetResponse<R>, Self::Error>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = self.url.join(method).map_err(HttpTransportError::Parse)?;
        debug!("Sending GET request to URL: {}", url);

        let mut request = self.client.get(url);
        for (name, value) in self.headers.iter() {
            request = request.header(name, value);
        }

        let response = request.send().await.map_err(HttpTransportError::Reqwest)?;

        let response_body = response.text().await.map_err(HttpTransportError::Reqwest)?;
        debug!("Response from GET request: {}", response_body);

        let parsed_response =
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
    async fn send_post_request<P, R>(
        &self,
        method: DevnetMethod,
        params: P,
    ) -> Result<DevnetResponse<R>, DevnetProviderError>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        Ok(self
            .transport
            .send_get_request(method)
            .await
            .map_err(DevnetClientError::TransportError)?)
    }
    async fn send_get_request<P, R>(&self, method: &str) -> Result<Config, DevnetProviderError>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        Ok(self
            .transport
            .send_get_request(method)
            .await
            .map_err(DevnetClientError::TransportError)?)
    }
}

impl<T> Provider for DevnetClient<T>
where
    T: 'static + DevnetTransport + Sync + Send,
{
    async fn get_config(&self) -> Result<Config, ProviderError> {
        self.send_get_request("/config").await
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

impl<T> From<DevnetClientError<T>> for DevnetProviderError
where
    T: 'static + Error + Send + Sync,
{
    fn from(value: DevnetClientError<T>) -> Self {
        Self::Other(Box::new(value))
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
