use crate::{
    assert_result,
    utils::v7::{
        accounts::account::ConnectedAccount, endpoints::errors::OpenRpcTestGenError,
        providers::provider::Provider,
    },
    RunnableTrait,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let erc20_eth_address =
            Felt::from_hex("049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")?;
        let key: Felt =
            Felt::from_hex("0000000000000000000000000000000000000000000000000000000000000001")?;

        let storage_value = test_input
            .random_paymaster_account
            .provider()
            .get_storage_at(erc20_eth_address, key, BlockId::Tag(BlockTag::Latest))
            .await;

        let result = storage_value.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
