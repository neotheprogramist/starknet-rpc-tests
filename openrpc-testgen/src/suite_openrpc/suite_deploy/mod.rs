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
pub mod suite_contract_calls;
pub mod test_invoke_txn_v1;
pub mod test_invoke_txn_v3;

#[derive(Clone, Debug)]
pub struct TestSuiteDeploy {
    pub random_paymaster_account: RandomSingleOwnerAccount,
    pub random_executable_account: RandomSingleOwnerAccount,
    pub declaration_result: ClassAndTxnHash<Felt>,
}

impl SetupableTrait for TestSuiteDeploy {
    type Input = super::TestSuiteOpenRpc;
    type Output = TestSuiteDeploy;

    async fn setup(setup_input: &Self::Input) -> Result<Self::Output, RpcError> {
        let (flattened_sierra_class, compiled_class_hash) =
            get_compiled_contract(
                PathBuf::from_str("target/dev/contracts_contracts_sample_contract_3_HelloStarknet.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_3_HelloStarknet.compiled_contract_class.json")?,
            )
            .await?;

        let declaration_result = setup_input
            .random_paymaster_account
            .declare_v3(flattened_sierra_class, compiled_class_hash)
            .send()
            .await?;

        Ok(TestSuiteDeploy {
            random_paymaster_account: setup_input.random_paymaster_account.clone(),
            random_executable_account: setup_input.random_executable_account.clone(),
            declaration_result,
        })
    }
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(
    env!("OUT_DIR"),
    "/generated_tests_suite_openrpc_suite_deploy.rs"
));
