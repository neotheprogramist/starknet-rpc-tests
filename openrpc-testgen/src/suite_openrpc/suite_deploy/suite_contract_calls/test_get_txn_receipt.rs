use crate::utils::v7::accounts::account::{Account, ConnectedAccount};
use crate::utils::v7::accounts::call::Call;
use crate::utils::v7::endpoints::utils::{get_selector_from_name, wait_for_sent_transaction};
use crate::utils::v7::providers::provider::Provider;
use crate::RandomizableAccountsTrait;
use crate::{utils::v7::endpoints::errors::RpcError, RunnableTrait};
use colored::Colorize;
use starknet_types_core::felt::Felt;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        let result = test_input
            .random_paymaster_account
            .execute_v3(vec![call])
            .send()
            .await?;

        wait_for_sent_transaction(
            result.transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        let receipt = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_receipt(result.transaction_hash)
            .await;

        match receipt {
            Ok(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc get_transaction_receipt COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_transaction_receipt INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
