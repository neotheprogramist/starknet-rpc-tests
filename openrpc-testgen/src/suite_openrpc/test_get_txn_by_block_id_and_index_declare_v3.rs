use std::{path::PathBuf, str::FromStr};

use crate::{
    utils::v7::{
        accounts::account::{Account, ConnectedAccount},
        endpoints::{
            declare_contract::get_compiled_contract, errors::RpcError,
            utils::wait_for_sent_transaction,
        },
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use colored::Colorize;
use starknet_types_rpc::{BlockId, DeclareTxn, MaybePendingBlockWithTxs, Txn};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_5_HelloStarknet.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_5_HelloStarknet.compiled_contract_class.json")?,
        )
        .await?;

        let declaration_result = test_input
            .random_paymaster_account
            .declare_v3(flattened_sierra_class, compiled_class_hash)
            .send()
            .await?;

        wait_for_sent_transaction(
            declaration_result.transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        let block_number = test_input
            .random_paymaster_account
            .provider()
            .block_hash_and_number()
            .await?
            .block_number;

        let block_with_txns = test_input
            .random_paymaster_account
            .provider()
            .get_block_with_txs(BlockId::Number(block_number))
            .await?;

        let txn_index: u64 = match block_with_txns {
            MaybePendingBlockWithTxs::Block(block_with_txs) => block_with_txs
                .transactions
                .iter()
                .position(|tx| tx.transaction_hash == declaration_result.transaction_hash)
                .ok_or_else(|| {
                    RpcError::TransactionNotFound(declaration_result.transaction_hash.to_string())
                })?
                .try_into()
                .map_err(|_| RpcError::TransactionIndexOverflow)?,
            MaybePendingBlockWithTxs::Pending(block_with_txs) => block_with_txs
                .transactions
                .iter()
                .position(|tx| tx.transaction_hash == declaration_result.transaction_hash)
                .ok_or_else(|| {
                    RpcError::TransactionNotFound(declaration_result.transaction_hash.to_string())
                })?
                .try_into()
                .map_err(|_| RpcError::TransactionIndexOverflow)?,
        };

        let txn = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_by_block_id_and_index(BlockId::Number(block_number), txn_index)
            .await?;

        match txn {
            Txn::Declare(DeclareTxn::V3(_)) => {
                info!(
                    "{} {}",
                    "✓ Rpc get_transaction_by_block_id_and_index_declare_v2 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            _ => {
                let error_message = format!("Unexpected transaction response type: {:?}", txn);
                error!(
                    "{} {} {}",
                    "✗ Rpc get_transaction_by_block_id_and_index_declare_v2 INCOMPATIBLE:".red(),
                    error_message,
                    "✗".red()
                );
                return Err(RpcError::UnexpectedTxnType(error_message));
            }
        }

        Ok(Self {})
    }
}
