use crate::{
    assert_result,
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::OpenRpcTestGenError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use starknet_types_rpc::{BlockId, BlockTag};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteDeploy;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let contract_class = test_input
            .random_paymaster_account
            .provider()
            .get_class(
                BlockId::Tag(BlockTag::Latest),
                test_input.declaration_result.class_hash,
            )
            .await;

        let result = contract_class.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
