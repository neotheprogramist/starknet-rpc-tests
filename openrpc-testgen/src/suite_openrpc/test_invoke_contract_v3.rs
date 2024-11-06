use colored::Colorize;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::TxnReceipt;
use tracing::{error, info};

use super::SetupOutput;
use crate::{
    utils::v7::{
        accounts::{
            account::{Account, AccountError, ConnectedAccount},
            call::Call,
        },
        contract::factory::ContractFactory,
        endpoints::{
            declare_contract::{
                extract_class_hash_from_error, get_compiled_contract, parse_class_hash_from_error,
                RunnerError,
            },
            errors::{CallError, RpcError},
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        providers::provider::{Provider, ProviderError},
    },
    RunnableTrait,
};

#[derive(Clone, Debug)]
pub struct TestCase {
    pub data: SetupOutput,
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
            .declare_v3(flattened_sierra_class, compiled_class_hash)
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

        let factory = ContractFactory::new(declaration_hash, self.data.paymaster_account.clone());
        let mut salt_buffer = [0u8; 32];
        let mut rng = StdRng::from_entropy();
        rng.fill_bytes(&mut salt_buffer[1..]);
        let deployment_result = factory
            .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .send()
            .await?;

        wait_for_sent_transaction(
            deployment_result.transaction_hash,
            &self.data.paymaster_account,
        )
        .await?;

        let deployment_receipt = self
            .data
            .paymaster_account
            .provider()
            .get_transaction_receipt(deployment_result.transaction_hash)
            .await?;

        let contract_address = match deployment_receipt {
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

        let increase_amount = Felt::from_hex("0x10")?;

        let call = Call {
            to: contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![increase_amount],
        };

        let invoke_contract_result = self
            .data
            .paymaster_account
            .execute_v3(vec![call])
            .send()
            .await;

        match invoke_contract_result {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc Invoke_contract_v1 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc Invoke_contract_v1 INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(())
    }
}
