use std::future::Future;
use utils::v7::{
    accounts::single_owner::SingleOwnerAccount,
    endpoints::errors::RpcError,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::local_wallet::LocalWallet,
};

pub mod macros;
// pub mod suite_openrpc;
pub mod suite_katana;
pub mod utils;

pub trait RunnableTrait: Sized {
    type Input;

    fn run(input: &Self::Input) -> impl Future<Output = Result<Self, RpcError>>;
}
pub trait SetupableTrait: Sized {
    type Input;

    fn setup(input: &Self::Input) -> impl Future<Output = Result<Self, RpcError>>;
}

pub trait RandomizableAccountsTrait {
    fn random_accounts(
        &self,
    ) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, RpcError>;
}
