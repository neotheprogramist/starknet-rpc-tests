use crate::{
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::RpcError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use colored::Colorize;
use starknet_types_core::felt::Felt;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let txn = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_by_hash(Felt::from_hex("0xdeadbeef")?)
            .await;

        match txn {
            Err(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc get_transaction_by_hash_non_existent COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Ok(_) => {
                error!(
                    "{} {}",
                    "✗ Rpc get_transaction_by_hash_non_existent INCOMPATIBLE:".red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
