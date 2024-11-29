use crate::{
    assert_result,
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::OpenRpcTestGenError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use starknet_types_rpc::BlockId;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let block_hash = test_input
            .random_paymaster_account
            .provider()
            .block_hash_and_number()
            .await?
            .block_hash;

        let block_txn_count = test_input
            .random_paymaster_account
            .provider()
            .get_block_transaction_count(BlockId::Hash(block_hash))
            .await?;

        let txn = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_by_block_id_and_index(BlockId::Hash(block_hash), block_txn_count - 1)
            .await;

        let result = txn.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
