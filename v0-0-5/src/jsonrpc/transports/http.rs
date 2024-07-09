use async_trait::async_trait;
use log::trace;
use reqwest::{Client, Url};
use serde::{de::DeserializeOwned, Serialize};
use tracing::info;
use url::ParseError;

use crate::jsonrpc::{
    transports::{JsonRpcTransport, JsonRpcTransportQueryParams},
    JsonRpcMethod, JsonRpcResponse,
};

#[derive(Debug, Clone)]
pub struct HttpTransport {
    client: Client,
    url: Url,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum HttpTransportError {
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    Parse(ParseError),
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T> {
    id: u64,
    jsonrpc: &'static str,
    method: JsonRpcMethod,
    params: T,
}

impl HttpTransport {
    pub fn new(url: impl Into<Url>) -> Self {
        Self::new_with_client(url, Client::new())
    }

    pub fn new_with_client(url: impl Into<Url>, client: Client) -> Self {
        Self {
            client,
            url: url.into(),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl JsonRpcTransport for HttpTransport {
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
        trace!("Sending request via JSON-RPC: {}", request_body);

        let response = self
            .client
            .post(self.url.clone())
            .body(request_body)
            .header("Content-Type", "application/json");

        let response = response.send().await.map_err(Self::Error::Reqwest)?;
        println!("{}", response.url());
        let response_body = response.text().await.map_err(Self::Error::Reqwest)?;
        trace!("Response from JSON-RPC: {}", response_body);

        let parsed_response = serde_json::from_str(&response_body).map_err(Self::Error::Json)?;

        Ok(parsed_response)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl JsonRpcTransportQueryParams for HttpTransport {
    type Error = HttpTransportError;

    async fn send_request_query_params<P>(
        &self,
        method: JsonRpcMethod,
        params: P,
    ) -> Result<serde_json::Value, Self::Error>
    where
        P: Serialize + Send + Sync,
    {
        let method_str = serde_json::to_string(&method).map_err(Self::Error::Json)?;
        let method_str = method_str.trim_matches('"').to_string();

        let url = match self.url.join(&method_str) {
            Ok(url) => url,
            Err(e) => return Err(HttpTransportError::Parse(e)),
        };

        let res = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await
            .map_err(Self::Error::Reqwest)?;
        let response_body = res.text().await.map_err(Self::Error::Reqwest)?;
        let parsed_response = serde_json::from_str(&response_body).map_err(Self::Error::Json)?;
        Ok(parsed_response)
    }
}
