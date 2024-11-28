use crate::{
    assert_result,
    utils::v7::{
        accounts::account::{Account, AccountError},
        endpoints::{
            declare_contract::{
                extract_class_hash_from_error, get_compiled_contract, parse_class_hash_from_error,
                RunnerError,
            },
            errors::OpenRpcTestGenError,
            utils::wait_for_sent_transaction,
        },
        providers::provider::ProviderError,
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use std::{path::PathBuf, str::FromStr, sync::Arc};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_1_HelloStarknet.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_1_HelloStarknet.compiled_contract_class.json")?,
        )
        .await?;

        let declaration_hash = match test_input
            .random_paymaster_account
            .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
            .send()
            .await
        {
            Ok(result) => {
                wait_for_sent_transaction(
                    result.transaction_hash,
                    &test_input.random_paymaster_account.random_accounts()?,
                )
                .await?;

                Ok(result.class_hash)
            }
            Err(AccountError::Signing(sign_error)) => {
                if sign_error.to_string().contains("is already declared") {
                    Ok(parse_class_hash_from_error(&sign_error.to_string())?)
                } else {
                    Err(OpenRpcTestGenError::RunnerError(
                        RunnerError::AccountFailure(format!(
                            "Transaction execution error: {}",
                            sign_error
                        )),
                    ))
                }
            }

            Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
                if starkneterror.to_string().contains("is already declared") {
                    Ok(parse_class_hash_from_error(&starkneterror.to_string())?)
                } else {
                    Err(OpenRpcTestGenError::RunnerError(
                        RunnerError::AccountFailure(format!(
                            "Transaction execution error: {}",
                            starkneterror
                        )),
                    ))
                }
            }
            Err(e) => {
                let full_error_message = format!("{:?}", e);

                if full_error_message.contains("is already declared") {
                    Ok(extract_class_hash_from_error(&full_error_message)?)
                } else {
                    let full_error_message = format!("{:?}", e);

                    return Err(OpenRpcTestGenError::AccountError(AccountError::Other(
                        full_error_message,
                    )));
                }
            }
        };

        let result = declaration_hash.is_ok();

        assert_result!(result);

        Ok(Self {})
    }
}
