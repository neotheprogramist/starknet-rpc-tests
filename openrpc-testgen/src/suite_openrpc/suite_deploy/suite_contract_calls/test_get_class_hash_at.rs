use crate::utils::v7::accounts::account::ConnectedAccount;
use crate::utils::v7::providers::provider::Provider;
use crate::{utils::v7::endpoints::errors::RpcError, RunnableTrait};
use colored::Colorize;
use starknet_types_rpc::{BlockId, BlockTag};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let contract_class_hash = test_input
            .random_paymaster_account
            .provider()
            .get_class_hash_at(
                BlockId::Tag(BlockTag::Latest),
                test_input.deployed_contract_address,
            )
            .await;

        match contract_class_hash {
            Ok(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc get_class_hash_at COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_class_hash_at INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
