use colored::Colorize;
use starknet::accounts::SingleOwnerAccount;
use tracing::{error, info};

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
        providers::{
            jsonrpc::{HttpTransport, JsonRpcClient},
            provider::ProviderError,
        },
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use std::{path::PathBuf, str::FromStr, sync::Arc};

use super::SetupOutput;

#[derive(Clone, Debug)]
pub struct TestCase {
    pub data: SetupOutput,
}

impl RunnableTrait for TestCase {
    type Output = ();
    async fn run(&self) -> Result<Self::Output, RpcError> {
        let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_1_HelloStarknet.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_1_HelloStarknet.compiled_contract_class.json")?,
        )
        .await?;

        let declaration_hash = match self
            .data
            .random_paymaster_accounts
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
        };

        match declaration_hash {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc Add_declare_transaction_v2 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc add_declare_transaction_v2 INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(())
    }
}
