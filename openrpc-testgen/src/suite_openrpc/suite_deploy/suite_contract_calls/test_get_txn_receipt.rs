use crate::assert_result;
use crate::utils::v7::accounts::account::{Account, ConnectedAccount};
use crate::utils::v7::accounts::call::Call;
use crate::utils::v7::endpoints::utils::{get_selector_from_name, wait_for_sent_transaction};
use crate::utils::v7::providers::provider::Provider;
use crate::RandomizableAccountsTrait;
use crate::{utils::v7::endpoints::errors::OpenRpcTestGenError, RunnableTrait};
use starknet_types_core::felt::Felt;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
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

        let result = receipt.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
