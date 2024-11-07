use crate::suite_openrpc::suite_deploy::suite_contract_calls::SetupOutput;
use crate::utils::v7::accounts::account::Account;
use crate::{
    utils::v7::{
        accounts::call::Call,
        endpoints::{errors::RpcError, utils::get_selector_from_name},
    },
    RunnableTrait,
};
use colored::Colorize;
use starknet_types_core::felt::Felt;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = SetupOutput;
    type Output = ();

    async fn run(test_input: Self::Input) -> Result<Self::Output, RpcError> {
        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        let invoke_increase_balance_result = test_input
            .random_paymaster_account
            .execute_v1(vec![increase_balance_call])
            .send()
            .await;

        match invoke_increase_balance_result {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc Invoke_contract_v1 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc Invoke_contract_v1 INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(())
    }
}
