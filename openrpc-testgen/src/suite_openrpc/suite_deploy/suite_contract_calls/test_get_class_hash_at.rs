use crate::assert_result;
use crate::utils::v7::accounts::account::ConnectedAccount;
use crate::utils::v7::providers::provider::Provider;
use crate::{utils::v7::endpoints::errors::OpenRpcTestGenError, RunnableTrait};
use starknet_types_rpc::{BlockId, BlockTag};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let contract_class_hash = test_input
            .random_paymaster_account
            .provider()
            .get_class_hash_at(
                BlockId::Tag(BlockTag::Latest),
                test_input.deployed_contract_address,
            )
            .await;

        let result = contract_class_hash.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
