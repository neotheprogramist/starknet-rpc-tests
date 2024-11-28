use crate::utils::v7::accounts::account::ConnectedAccount;
use crate::utils::v7::endpoints::errors::CallError;
use crate::utils::v7::providers::provider::Provider;
use crate::{utils::v7::endpoints::errors::OpenRpcTestGenError, RunnableTrait};
use colored::Colorize;
use starknet_types_rpc::TxnReceipt;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteContractCalls;

    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let tx_hash = match &test_input.deployment_receipt {
            TxnReceipt::Deploy(receipt) => receipt.common_receipt_properties.transaction_hash,
            TxnReceipt::Invoke(receipt) => receipt.common_receipt_properties.transaction_hash,
            _ => {
                return Err(OpenRpcTestGenError::CallError(
                    CallError::UnexpectedReceiptType,
                ));
            }
        };

        let tx_status = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_status(tx_hash)
            .await;

        match tx_status {
            Ok(_) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc get_transaction_status COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_transaction_status INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(Self {})
    }
}
