use crate::{
    utils::v7::{contract::factory::ContractFactory, endpoints::errors::RpcError},
    RandomizableAccountsTrait, RunnableTrait,
};
use colored::Colorize;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_types_core::felt::Felt;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteDeploy;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
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

        match invoke_result {
            Ok(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc deploy_transaction_v3 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc deploy_transaction_v3 INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
