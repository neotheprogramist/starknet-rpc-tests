use std::fmt::Debug;

use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_types_core::felt::Felt;
use starknet_types_core::felt::FromStrError;
use starknet_types_rpc::v0_5_0::AddInvokeTransactionResult;

use thiserror::Error;

use crate::v5::rpc::{
    accounts::single_owner::SingleOwnerAccount,
    contract::factory::ContractFactory,
    providers::provider::{Provider, ProviderError},
    signers::local_wallet::LocalWallet,
};

use super::declare_contract::RunnerError;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum DeployError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),
}
#[allow(dead_code)]
pub async fn deploy_contract<P: Provider + Send + Sync + Debug>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    class_hash: Felt,
) -> AddInvokeTransactionResult {
    let factory = ContractFactory::new(class_hash, account);
    let mut salt_buffer = [0u8; 32];
    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut salt_buffer[1..]);

    factory
        .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
        .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
        .send()
        .await
        .unwrap()
}
