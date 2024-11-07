use super::SetupOutput;
use crate::{
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::RpcError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use colored::Colorize;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = SetupOutput;
    type Output = ();

    async fn run(test_input: Self::Input) -> Result<Self::Output, RpcError> {
        let chain_id = test_input
            .random_paymaster_account
            .provider()
            .chain_id()
            .await;

        match chain_id {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc get_chain_id COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_chain_id INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(())
    }
}
