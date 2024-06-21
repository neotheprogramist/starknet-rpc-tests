use std::sync::Arc;

use starknet_crypto::FieldElement;
use starknet_signers::LocalWallet;
use utils::{
    account::{single_owner::SingleOwnerAccount, Account, AccountError},
    errors::{parse_class_hash_from_error, RunnerError},
    provider::{Provider, ProviderError},
    starknet_utils::get_compiled_contract,
};

pub async fn declare_contract_v3<P: Provider + Send + Sync>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    sierra_path: &str,
    casm_path: &str,
) -> Result<FieldElement, RunnerError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    match account
        .declare_v3(Arc::new(flattened_sierra_class), compiled_class_hash)
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                )))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
            } else {
                Err(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    starkneterror
                )))
            }
        }
        Err(e) => {
            tracing::info!("General account error encountered: {:?}, possible cause - incorrect address or public_key in environment variables!", e);
            Err(RunnerError::AccountFailure(format!("Account error: {}", e)))
        }
    }
}
