pub mod http;

use auto_impl::auto_impl;
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

pub use http::HttpTransport;

use crate::v6::providers::jsonrpc::{JsonRpcMethod, JsonRpcResponse};

#[auto_impl(&, Box, Arc)]
pub trait JsonRpcTransport {
    type Error: Error + Send + Sync;

    fn send_request<P, R>(
        &self,
        method: JsonRpcMethod,
        params: P,
    ) -> impl std::future::Future<Output = Result<JsonRpcResponse<R>, Self::Error>> + Send
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned;
}
