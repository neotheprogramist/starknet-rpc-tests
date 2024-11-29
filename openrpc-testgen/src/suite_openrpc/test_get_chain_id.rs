use crate::{
    assert_result,
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::OpenRpcTestGenError,
        providers::provider::Provider,
    },
    RunnableTrait,
};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let chain_id = test_input
            .random_paymaster_account
            .provider()
            .chain_id()
            .await;

        let result = chain_id.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
