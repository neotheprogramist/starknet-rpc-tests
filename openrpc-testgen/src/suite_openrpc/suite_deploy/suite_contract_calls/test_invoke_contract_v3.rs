use crate::utils::v7::accounts::account::Account;
use crate::utils::v7::endpoints::utils::wait_for_sent_transaction;
use crate::RandomizableAccountsTrait;
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
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        let invoke_result = test_input
            .random_paymaster_account
            .execute_v3(vec![increase_balance_call])
            .send()
            .await;

        wait_for_sent_transaction(
            invoke_result.as_ref().unwrap().transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        match invoke_result {
            Ok(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc invoke_contract_v3 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc invoke_contract_v3 INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
