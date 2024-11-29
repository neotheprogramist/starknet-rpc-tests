use crate::{
    assert_matches_result, assert_result,
    utils::v7::{
        accounts::account::{Account, AccountError, ConnectedAccount},
        endpoints::{
            declare_contract::get_compiled_contract, errors::OpenRpcTestGenError,
            utils::wait_for_sent_transaction,
        },
        providers::{
            jsonrpc::StarknetError,
            provider::{Provider, ProviderError},
        },
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag};
use std::{path::PathBuf, str::FromStr, sync::Arc};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_2_HelloStarknet.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_2_HelloStarknet.compiled_contract_class.json")?,
        )
        .await?;
        let provider = test_input
            .random_paymaster_account
            .random_accounts()?
            .provider()
            .clone();

        let declare_res = test_input
            .random_paymaster_account
            .declare_v2(
                Arc::new(flattened_sierra_class.clone()),
                compiled_class_hash,
            )
            .send()
            .await?;

        let (transaction_hash, class_hash) = (declare_res.transaction_hash, declare_res.class_hash);

        wait_for_sent_transaction(
            transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        let get_class = provider
            .clone()
            .get_class(BlockId::Tag(BlockTag::Pending), class_hash)
            .await
            .is_ok();

        assert_result!(get_class);

        let declare_result = test_input
            .random_paymaster_account
            .declare_v2(
                Arc::new(flattened_sierra_class.clone()),
                compiled_class_hash,
            )
            .max_fee(Felt::ZERO) //TODO: without max_fee diffrent error type than ClassAlreadyDeclared starknet devnet: Provider(StarknetError(TransactionExecutionError(TransactionExecutionErrorData
            .send()
            .await;

        assert_matches_result!(
            declare_result.unwrap_err(),
            AccountError::Provider(ProviderError::StarknetError(
                StarknetError::ClassAlreadyDeclared
            ))
        );

        Ok(Self {})
    }
}
