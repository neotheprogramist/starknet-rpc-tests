use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{ClassAndTxnHash, TxnReceipt};

use super::RandomSingleOwnerAccount;
use crate::{
    utils::v7::{
        accounts::account::ConnectedAccount,
        contract::factory::ContractFactory,
        endpoints::errors::{CallError, RpcError},
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, SetupableTrait,
};

pub mod test_invoke_contract_v1;

pub struct TestSuiteContractCalls {
    pub random_paymaster_accounts: RandomSingleOwnerAccount,
    pub random_executable_accounts: RandomSingleOwnerAccount,
    pub declaration_result: ClassAndTxnHash<Felt>,
}

#[derive(Clone, Debug)]
pub struct SetupOutput {
    pub random_paymaster_accounts: RandomSingleOwnerAccount,
    pub random_executable_accounts: RandomSingleOwnerAccount,
    pub deployed_contract_address: Felt,
}

impl SetupableTrait for TestSuiteContractCalls {
    type Output = SetupOutput;

    async fn setup(&self) -> Result<Self::Output, RpcError> {
        let factory = ContractFactory::new(
            self.declaration_result.class_hash,
            self.random_paymaster_accounts.random_accounts()?,
        );
        let mut salt_buffer = [0u8; 32];
        let mut rng = StdRng::from_entropy();
        rng.fill_bytes(&mut salt_buffer[1..]);

        let deployment_result = factory
            .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .send()
            .await?;

        let deployment_receipt = self
            .random_paymaster_accounts
            .provider()
            .get_transaction_receipt(deployment_result.transaction_hash)
            .await?;

        let deployed_contract_address = match deployment_receipt {
            TxnReceipt::Deploy(receipt) => receipt.contract_address,
            TxnReceipt::Invoke(receipt) => {
                if let Some(contract_address) = receipt
                    .common_receipt_properties
                    .events
                    .first()
                    .and_then(|event| event.data.first())
                {
                    *contract_address
                } else {
                    return Err(RpcError::CallError(CallError::UnexpectedReceiptType));
                }
            }
            _ => {
                return Err(RpcError::CallError(CallError::UnexpectedReceiptType));
            }
        };

        Ok(SetupOutput {
            random_paymaster_accounts: self.random_paymaster_accounts.clone(),
            random_executable_accounts: self.random_executable_accounts.clone(),
            deployed_contract_address,
        })
    }
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(
    env!("OUT_DIR"),
    "/generated_tests_suite_openrpc_suite_deploy_suite_contract_calls.rs"
));
