use std::time::Duration;

use crate::utils::v7::accounts::account::Account;
use crate::utils::v7::accounts::account::ConnectedAccount;
use crate::utils::v7::accounts::call::Call;
use crate::utils::v7::accounts::single_owner::SingleOwnerAccount;
use crate::utils::v7::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use crate::utils::v7::providers::provider::Provider;
use crate::utils::v7::signers::local_wallet::LocalWallet;
use crate::utils::v7::{
    accounts::account::{normalize_address, starknet_keccak},
    contract::{CompiledClass, HashAndFlatten, SierraClass},
    endpoints::errors::RpcError,
};
use reqwest::Client;
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::{Pedersen, StarkHash};
use starknet_types_rpc::v0_7_1::{ContractClass, TxnHash};
use starknet_types_rpc::{
    BlockId, BlockTag, TxnExecutionStatus, TxnFinalityAndExecutionStatus, TxnStatus,
};
use tokio::io::AsyncReadExt;
use tracing::debug;
use tracing::{error, warn};
use url::Url;

use super::{declare_contract::RunnerError, errors::NonAsciiNameError};

const DEFAULT_ENTRY_POINT_NAME: &str = "__default__";
const DEFAULT_L1_ENTRY_POINT_NAME: &str = "__l1_default__";

pub async fn get_compiled_contract(
    sierra_path: &str,
    casm_path: &str,
) -> Result<(ContractClass<Felt>, TxnHash<Felt>), RunnerError> {
    let mut file = tokio::fs::File::open(sierra_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut sierra = String::default();
    file.read_to_string(&mut sierra)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let mut file = tokio::fs::File::open(casm_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut casm = String::default();
    file.read_to_string(&mut casm)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let contract_artifact: SierraClass = serde_json::from_str(&sierra)?;

    let compiled_class: CompiledClass = serde_json::from_str(&casm)?;

    let casm_class_hash = compiled_class.class_hash().unwrap();

    let flattened_class = contract_artifact.clone().flatten().unwrap();

    Ok((flattened_class, casm_class_hash))
}

pub async fn restart_devnet(url: Url) -> Result<(), RpcError> {
    let client = Client::new();
    let url = url.join("/restart")?;
    let response = client.post(url).send().await?;
    if response.status().is_success() {
        debug!("Devnet restarted successfully.");
        Ok(())
    } else {
        error!("Failed to restart Devnet. Status: {}", response.status());
        Err(RpcError::RequestError(
            response.error_for_status().unwrap_err(),
        ))
    }
}

pub fn get_selector_from_name(func_name: &str) -> Result<Felt, NonAsciiNameError> {
    if func_name == DEFAULT_ENTRY_POINT_NAME || func_name == DEFAULT_L1_ENTRY_POINT_NAME {
        Ok(Felt::ZERO)
    } else {
        let name_bytes = func_name.as_bytes();
        if name_bytes.is_ascii() {
            Ok(starknet_keccak(name_bytes))
        } else {
            Err(NonAsciiNameError)
        }
    }
}

#[allow(dead_code)]
pub fn get_storage_var_address(var_name: &str, args: &[Felt]) -> Result<Felt, NonAsciiNameError> {
    let var_name_bytes = var_name.as_bytes();
    if var_name_bytes.is_ascii() {
        let mut res = starknet_keccak(var_name_bytes);
        for arg in args.iter() {
            res = Pedersen::hash(&res, arg);
        }
        Ok(normalize_address(res))
    } else {
        Err(NonAsciiNameError)
    }
}

pub fn validate_inputs(
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<(Felt, Felt, Felt, Felt, Felt), RpcError> {
    match (
        account_address,
        private_key,
        erc20_strk_contract_address,
        erc20_eth_contract_address,
        amount_per_test,
    ) {
        (
            Some(account_address),
            Some(private_key),
            Some(erc20_strk_contract_address),
            Some(erc20_eth_contract_address),
            Some(amount_per_test),
        ) => {
            if amount_per_test <= Felt::ZERO {
                warn!("Amount per test must be greater than zero");
                return Err(RpcError::InvalidInput(
                    "Amount per test must be greater than zero".to_string(),
                ));
            };
            Ok((
                account_address,
                private_key,
                erc20_strk_contract_address,
                erc20_eth_contract_address,
                amount_per_test,
            ))
        }
        (None, _, _, _, _) => {
            warn!("Account address is required");
            Err(RpcError::InvalidInput(
                "Account address is required".to_string(),
            ))
        }
        (_, None, _, _, _) => {
            warn!("Private key is required to fund generated account");
            Err(RpcError::InvalidInput(
                "Private key is required".to_string(),
            ))
        }
        (_, _, None, _, _) => {
            warn!("ERC20 STRK contract address is required");
            Err(RpcError::InvalidInput(
                "ERC20 STRK contract address is required".to_string(),
            ))
        }
        (_, _, _, None, _) => {
            warn!("ERC20 ETH contract address is required");
            Err(RpcError::InvalidInput(
                "ERC20 ETH contract address is required".to_string(),
            ))
        }
        (_, _, _, _, None) => {
            warn!("Amount per test is required");
            Err(RpcError::InvalidInput(
                "Amount per test is required".to_string(),
            ))
        }
    }
}

use starknet_types_rpc::MaybePendingBlockWithTxHashes;
pub async fn wait_for_sent_transaction(
    transaction_hash: Felt,
    user_passed_account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<TxnFinalityAndExecutionStatus, RpcError> {
    let start_fetching = std::time::Instant::now();
    let wait_for = Duration::from_secs(60);

    loop {
        if start_fetching.elapsed() > wait_for {
            return Err(RpcError::Timeout(
                "Transaction not mined in 60 seconds.".to_string(),
            ));
        }

        // Check transaction status
        let status = match user_passed_account
            .provider()
            .get_transaction_status(transaction_hash)
            .await
        {
            Ok(status) => status,
            Err(_e) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        match status {
            TxnFinalityAndExecutionStatus {
                finality_status: TxnStatus::AcceptedOnL2,
                execution_status: Some(TxnExecutionStatus::Succeeded),
                ..
            } => {
                debug!(
                    "Transaction status: AcceptedOnL2 and Succeeded. Checking block inclusion..."
                );

                // Check if the transaction is in the pending block
                let in_pending = match user_passed_account
                    .provider()
                    .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Pending))
                    .await
                {
                    Ok(MaybePendingBlockWithTxHashes::Pending(block)) => {
                        block.transactions.contains(&transaction_hash)
                    }
                    _ => false,
                };

                // Check if the transaction is in the latest block
                let in_latest = match user_passed_account
                    .provider()
                    .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
                    .await
                {
                    Ok(MaybePendingBlockWithTxHashes::Block(block)) => {
                        block.transactions.contains(&transaction_hash)
                    }
                    _ => false,
                };

                if in_pending && !in_latest {
                    debug!(
                        "Transaction is in Pending block but not yet in Latest block. Retrying..."
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }

                if in_latest && !in_pending {
                    debug!(
                        "✅ Transaction confirmed in Latest block and not in Pending. Finishing..."
                    );
                    return Ok(status);
                }

                debug!("Transaction is neither in Latest nor finalized. Retrying...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            TxnFinalityAndExecutionStatus {
                finality_status: TxnStatus::AcceptedOnL2,
                execution_status: Some(TxnExecutionStatus::Reverted),
                ..
            } => {
                debug!("❌ Transaction reverted on L2. Stopping...");
                return Err(RpcError::TransactionFailed(transaction_hash.to_string()));
            }
            TxnFinalityAndExecutionStatus {
                finality_status: TxnStatus::Rejected,
                ..
            } => {
                debug!("❌ Transaction rejected. Stopping...");
                return Err(RpcError::TransactionRejected(transaction_hash.to_string()));
            }
            _ => {
                debug!("Transaction status not finalized. Retrying...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
    }
}

pub async fn setup_generated_account(
    mut user_passed_account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    erc20_eth_contract_address: Felt,
    erc20_strk_contract_address: Felt,
    amount_per_test: Felt,
    create_acc_data_address: Felt,
) -> Result<(), RpcError> {
    user_passed_account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let transfer_execution = user_passed_account
        .execute_v3(vec![
            Call {
                to: erc20_strk_contract_address,
                selector: get_selector_from_name("transfer")?,
                calldata: vec![create_acc_data_address, amount_per_test, Felt::ZERO],
            },
            Call {
                to: erc20_eth_contract_address,
                selector: get_selector_from_name("transfer")?,
                calldata: vec![create_acc_data_address, amount_per_test, Felt::ZERO],
            },
        ])
        .send()
        .await?;

    wait_for_sent_transaction(transfer_execution.transaction_hash, &user_passed_account).await?;
    Ok(())
}
