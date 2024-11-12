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
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let block_txs = test_input
            .random_paymaster_account
            .provider()
            .get_block_with_txs(BlockId::Tag(BlockTag::Latest))
            .await;

        match block_txs {
            Ok(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc get_block_with_txs COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_block_with_txs INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
