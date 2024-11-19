use std::{path::PathBuf, str::FromStr};

use crate::{
    utils::v7::{
        accounts::account::{Account, AccountError, ConnectedAccount},
        endpoints::{
            declare_contract::{
                extract_class_hash_from_error, get_compiled_contract, parse_class_hash_from_error,
                RunnerError,
            },
            errors::RpcError,
            utils::wait_for_sent_transaction,
            utils::wait_for_sent_transaction,
        },
        providers::provider::{Provider, ProviderError},
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use colored::Colorize;
use starknet_types_rpc::{
    BlockId, DeclareTxn, EventFilterWithPageRequest, MaybePendingBlockWithTxs, Txn,
};
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

        let (declaration_tx_hash, tx_block) = match test_input
            .random_paymaster_account
            .declare_v3(flattened_sierra_class, compiled_class_hash)
            .send()
            .await
        {
            Ok(result) => {
                wait_for_sent_transaction(
                    result.transaction_hash,
                    &test_input.random_executable_account.random_accounts()?,
                )
                .await?;
                let block_number = test_input
                    .random_paymaster_account
                    .provider()
                    .block_hash_and_number()
                    .await?
                    .block_number;
                Ok((result.transaction_hash, block_number))
            }
            Err(AccountError::Signing(sign_error)) => {
                if sign_error.to_string().contains("is already declared") {
                    Ok((parse_class_hash_from_error(&sign_error.to_string())?, 0))
                } else {
                    Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                        "Transaction execution error: {}",
                        sign_error
                    ))))
                }
            }
            Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
                if starkneterror.to_string().contains("is already declared") {
                    Ok((parse_class_hash_from_error(&starkneterror.to_string())?, 0))
                } else {
                    Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                        "Transaction execution error: {}",
                        starkneterror
                    ))))
                }
            }
            Err(e) => {
                let full_error_message = format!("{:?}", e);
                if full_error_message.contains("is already declared") {
                    let class_hash = extract_class_hash_from_error(&full_error_message)?;

                    let filter = EventFilterWithPageRequest {
                        address: None,
                        from_block: Some(BlockId::Number(322392)),
                        to_block: Some(BlockId::Number(322392)),
                        keys: Some(vec![vec![]]),
                        chunk_size: 100,
                        continuation_token: None,
                    };
                    let paymaster_account_address = test_input
                        .random_paymaster_account
                        .random_accounts()?
                        .address();

                    let mut continuation_token = None;
                    let mut found_txn_hash = None;
                    let mut found_block_number = None;

                    loop {
                        let mut current_filter = filter.clone();
                        current_filter.continuation_token = continuation_token.clone();

                        let events_chunk = test_input
                            .random_paymaster_account
                            .provider()
                            .get_events(current_filter)
                            .await?;

                        for event in events_chunk.events {
                            if event.event.data.contains(&paymaster_account_address) {
                                let txn_hash = event.transaction_hash;

                                let txn_details = test_input
                                    .random_paymaster_account
                                    .provider()
                                    .get_transaction_by_hash(txn_hash)
                                    .await?;

                                if let Txn::Declare(DeclareTxn::V3(declare_txn)) = txn_details {
                                    if declare_txn.class_hash == class_hash {
                                        found_txn_hash = Some(txn_hash);
                                        found_block_number = event.block_number;
                                        break;
                                    }
                                }
                            }
                        }

                        if found_txn_hash.is_some() && found_block_number.is_some() {
                            break;
                        }

                        if let Some(token) = events_chunk.continuation_token {
                            continuation_token = Some(token);
                        } else {
                            break;
                        }
                    }

                    if let (Some(tx_hash), Some(block_number)) =
                        (found_txn_hash, found_block_number)
                    {
                        Ok((tx_hash, block_number))
                    } else {
                        Err(RpcError::RunnerError(RunnerError::AccountFailure(
                            "Transaction hash not found for the declared class.".to_string(),
                        )))
                    }
                } else {
                    Err(RpcError::AccountError(AccountError::Other(
                        full_error_message,
                    )))
                }
            }
        }?;

        let block_with_txns = test_input
            .random_paymaster_account
            .provider()
            .get_block_with_txs(BlockId::Number(tx_block))
            .await?;

        let txn_index: u64 = match block_with_txns {
            MaybePendingBlockWithTxs::Block(block_with_txs) => block_with_txs
                .transactions
                .iter()
                .position(|tx| tx.transaction_hash == declaration_tx_hash)
                .ok_or_else(|| RpcError::TransactionNotFound(declaration_tx_hash.to_string()))?
                .try_into()
                .map_err(|_| RpcError::TransactionIndexOverflow)?,
            MaybePendingBlockWithTxs::Pending(block_with_txs) => block_with_txs
                .transactions
                .iter()
                .position(|tx| tx.transaction_hash == declaration_tx_hash)
                .ok_or_else(|| RpcError::TransactionNotFound(declaration_tx_hash.to_string()))?
                .try_into()
                .map_err(|_| RpcError::TransactionIndexOverflow)?,
        };

        let txn = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_by_block_id_and_index(BlockId::Number(tx_block), txn_index)
            .await?;

        match txn {
            Txn::Declare(DeclareTxn::V3(_)) => {
                info!(
                    "{} {}",
                    "\n✓ Rpc get_transaction_by_block_id_and_index_declare_v3 COMPATIBLE".green(),
                    "✓".green()
                );
            }
            _ => {
                let error_message = format!("Unexpected transaction response type: {:?}", txn);
                error!(
                    "{} {} {}",
                    "✗ Rpc get_transaction_by_block_id_and_index_declare_v3 INCOMPATIBLE:".red(),
                    error_message,
                    "✗".red()
                );
                return Err(RpcError::UnexpectedTxnType(error_message));
            }
        }

        Ok(Self {})
    }
}
