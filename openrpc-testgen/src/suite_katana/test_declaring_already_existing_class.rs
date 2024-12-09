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

        let account = test_input.random_paymaster_account.random_accounts()?;
        let provider = account.provider().clone();

        // Declare the class for the first time.
        let declare_res = account
            .declare_v2(
                Arc::new(flattened_sierra_class.clone()),
                compiled_class_hash,
            )
            .send()
            .await?;

        let (transaction_hash, class_hash) = (declare_res.transaction_hash, declare_res.class_hash);

        wait_for_sent_transaction(transaction_hash, &account).await?;

        // check that the class is actually declared
        let get_class = provider
            .clone()
            .get_class(BlockId::Tag(BlockTag::Pending), class_hash)
            .await
            .is_ok();

        assert_result!(get_class);

        // -----------------------------------------------------------------------
        // Declaring the same class again should fail with a ClassAlreadyDeclared error

        // We set max fee manually to avoid perfoming fee estimation as we just want to test that the
        // pool validation will reject the tx.
        //
        // The value of the max fee is also irrelevant here, as the validator will only perform static
        // checks and will not run the account's validation.

        let declare_result = account
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
