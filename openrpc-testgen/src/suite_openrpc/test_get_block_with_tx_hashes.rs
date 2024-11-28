use crate::{
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::OpenRpcTestGenError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use colored::Colorize;
use starknet_types_rpc::{BlockId, BlockTag};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

// TODO create two cases - pending / latest
impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let block_with_tx_hashes = test_input
            .random_paymaster_account
            .provider()
            .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
            .await;

        match block_with_tx_hashes {
            Ok(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc get_block_with_tx_hashes COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_block_with_tx_hashes INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
