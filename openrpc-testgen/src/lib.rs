use std::future::Future;
use utils::v7::{
    accounts::single_owner::SingleOwnerAccount,
    endpoints::errors::RpcError,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::local_wallet::LocalWallet,
};

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

pub trait RandomizableAccountsTrait {
    fn random_accounts(
        &self,
    ) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, RpcError>;
}
