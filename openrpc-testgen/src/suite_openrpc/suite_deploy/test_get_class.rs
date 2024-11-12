use crate::{
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::RpcError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use colored::Colorize;
use starknet_types_rpc::{BlockId, BlockTag};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteDeploy;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let contract_class = test_input
            .random_paymaster_account
            .provider()
            .get_class(
                BlockId::Tag(BlockTag::Latest),
                test_input.declaration_result.class_hash,
            )
            .await;

        match contract_class {
            Ok(_) => {
                info!("{} {}", "\n✓ Rpc get_class COMPATIBLE".green(), "✓".green());
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_class INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
