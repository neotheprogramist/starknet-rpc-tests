use crate::{
    assert_result,
    utils::v7::{
        contract::factory::ContractFactory,
        endpoints::{errors::OpenRpcTestGenError, utils::wait_for_sent_transaction},
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_types_core::felt::Felt;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteDeploy;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let factory = ContractFactory::new(
            test_input.declaration_result.class_hash,
            test_input.random_paymaster_account.random_accounts()?,
        );
        let mut salt_buffer = [0u8; 32];
        let mut rng = StdRng::from_entropy();
        rng.fill_bytes(&mut salt_buffer[1..]);

        let invoke_result = factory
            .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .send()
            .await;

        wait_for_sent_transaction(
            invoke_result.as_ref().unwrap().transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        let result = invoke_result.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
