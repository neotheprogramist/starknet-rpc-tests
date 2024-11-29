use crate::{
    assert_result,
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::OpenRpcTestGenError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use starknet_types_core::felt::Felt;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let txn = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_by_hash(Felt::from_hex("0xdeadbeef")?)
            .await;

        let result = txn.is_err();

        assert_result!(result);

        Ok(Self {})
    }
}
