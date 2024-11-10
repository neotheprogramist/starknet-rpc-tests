use crate::utils::v7::accounts::account::ConnectedAccount;
use crate::utils::v7::providers::provider::Provider;
use crate::{
    utils::v7::endpoints::{errors::RpcError, utils::get_selector_from_name},
    RunnableTrait,
};
use colored::Colorize;
use starknet_types_rpc::{BlockId, BlockTag, MsgFromL1};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let estimate = test_input
            .random_paymaster_account
            .provider()
            .estimate_message_fee(
                MsgFromL1 {
                    from_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                    to_address: test_input.deployed_contract_address,
                    entry_point_selector: get_selector_from_name("deposit")?,
                    payload: vec![(1_u32).into(), (10_u32).into()],
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await;

        match estimate {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc estimate_message_fee COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc estimate_message_fee INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
