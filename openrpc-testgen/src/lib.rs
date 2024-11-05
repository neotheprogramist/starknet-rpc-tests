use std::future::Future;
use utils::v7::endpoints::errors::RpcError;

pub mod structs;
pub mod suite_openrpc;
pub mod utils;

pub trait RunnableTrait {
    type Output;

    fn run(&self) -> impl Future<Output = Result<Self::Output, RpcError>>;
}
pub trait SetupableTrait {
    type Output;

    fn setup(&self) -> impl Future<Output = Result<Self::Output, RpcError>>;
}
