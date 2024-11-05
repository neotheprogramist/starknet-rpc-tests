use starknet_types_core::felt::Felt;
use tracing::{info, warn};

use crate::{
    utils::v7::{
        accounts::account::{Account, AccountError},
        endpoints::{
            declare_contract::{
                extract_class_hash_from_error, get_compiled_contract, parse_class_hash_from_error,
                RunnerError,
            },
            errors::RpcError,
        },
        providers::provider::ProviderError,
    },
    RunnableTrait,
};
use std::{path::PathBuf, sync::Arc};

use super::SetupOutput;

#[derive(Clone, Debug)]
pub struct TestCase {
    pub data: SetupOutput,
}

#[derive(Clone, Debug)]
pub struct ContractPathPair {
    pub sierra_path: PathBuf,
    pub casm_path: PathBuf,
}

impl RunnableTrait for TestCase {
    type Output = ();
    async fn run(&self) -> Result<Self::Output, RpcError> {
        let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
            self.data.contracts_to_deploy_paths[0].sierra_path.clone(),
            self.data.contracts_to_deploy_paths[0].casm_path.clone(),
        )
        .await?;

        let declaration_hash = match self
            .data
            .paymaster_account
            .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
            .send()
            .await
        {
            Ok(result) => Ok(result.class_hash),
            Err(AccountError::Signing(sign_error)) => {
                if sign_error.to_string().contains("is already declared") {
                    Ok(parse_class_hash_from_error(&sign_error.to_string())?)
                } else {
                    Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                        "Transaction execution error: {}",
                        sign_error
                    ))))
                }
            }

            Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
                if starkneterror.to_string().contains("is already declared") {
                    Ok(parse_class_hash_from_error(&starkneterror.to_string())?)
                } else {
                    Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                        "Transaction execution error: {}",
                        starkneterror
                    ))))
                }
            }
            Err(e) => {
                let full_error_message = format!("{:?}", e);
                Ok(extract_class_hash_from_error(&full_error_message)?)
            }
        }?;

        if declaration_hash > Felt::ZERO {
            info!("GOOD TEST");
        } else {
            warn!("BAD TEST");
        }

        Ok(())
    }
}
