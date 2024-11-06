use std::path::PathBuf;

use starknet_types_core::felt::Felt;
use starknet_types_rpc::ClassAndTxnHash;

use super::RandomSingleOwnerAccount;
use crate::{
    utils::v7::{
        accounts::account::Account,
        endpoints::{declare_contract::get_compiled_contract, errors::RpcError},
    },
    SetupableTrait,
};
use std::str::FromStr;
pub mod test_invoke_txn_v1;

pub struct TestSuiteDeploy {
    random_paymaster_accounts: RandomSingleOwnerAccount,
    random_executable_accounts: RandomSingleOwnerAccount,
}

#[derive(Clone, Debug)]
pub struct SetupOutput {
    random_paymaster_accounts: RandomSingleOwnerAccount,
    random_executable_accounts: RandomSingleOwnerAccount,
    declaration_result: ClassAndTxnHash<Felt>,
}

impl SetupableTrait for TestSuiteDeploy {
    type Output = SetupOutput;

    async fn setup(&self) -> Result<Self::Output, RpcError> {
        let (flattened_sierra_class, compiled_class_hash) =
            get_compiled_contract(
                PathBuf::from_str("target/dev/contracts_contracts_sample_contract_3_HelloStarknet.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_3_HelloStarknet.compiled_contract_class.json")?,
            )
            .await?;

        let declaration_result = self
            .random_paymaster_accounts
            .declare_v3(flattened_sierra_class, compiled_class_hash)
            .send()
            .await?;

        Ok(SetupOutput {
            random_paymaster_accounts: self.random_paymaster_accounts.clone(),
            random_executable_accounts: self.random_executable_accounts.clone(),
            declaration_result,
        })
    }
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(
    env!("OUT_DIR"),
    "/generated_tests_suite_openrpc_suite_deploy.rs"
));
