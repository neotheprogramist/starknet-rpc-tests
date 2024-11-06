use colored::Colorize;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_types_core::felt::Felt;
use tracing::{error, info};

use super::SetupOutput;
use crate::{
    utils::v7::{contract::factory::ContractFactory, endpoints::errors::RpcError},
    RandomizableAccountsTrait, RunnableTrait,
};

#[derive(Clone, Debug)]
pub struct TestCase {
    pub data: SetupOutput,
}

impl RunnableTrait for TestCase {
    type Output = ();
    async fn run(&self) -> Result<Self::Output, RpcError> {
        let factory = ContractFactory::new(
            self.data.declaration_result.class_hash,
            self.data.random_paymaster_accounts.random_accounts()?,
        );
        let mut salt_buffer = [0u8; 32];
        let mut rng = StdRng::from_entropy();
        rng.fill_bytes(&mut salt_buffer[1..]);

        let invoke_result = factory
            .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .send()
            .await;

        match invoke_result {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc Add_invoke_transaction_v1 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc Add_invoke_v1 INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(())
    }
}
