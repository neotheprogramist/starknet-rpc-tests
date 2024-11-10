use crate::utils::v7::accounts::account::{Account, ConnectedAccount};
use crate::utils::v7::providers::provider::Provider;
use crate::{
    utils::v7::{
        accounts::call::Call,
        endpoints::{errors::RpcError, utils::get_selector_from_name},
    },
    RunnableTrait,
};
use colored::Colorize;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag, FunctionCall};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        test_input
            .random_paymaster_account
            .execute_v3(vec![increase_balance_call])
            .send()
            .await?;

        let balance = test_input
            .random_paymaster_account
            .provider()
            .call(
                FunctionCall {
                    calldata: vec![],
                    contract_address: test_input.deployed_contract_address,
                    entry_point_selector: get_selector_from_name("get_balance")?,
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await;

        match balance {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc call_contract COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc call_contract INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
