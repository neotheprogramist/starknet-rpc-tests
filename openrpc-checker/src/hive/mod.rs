use colored::*;
use starknet_types_core::felt::Felt;
use tracing::{error, info};
use url::Url;
use utils::random_url::{get_random_url, set_random_rpc_url};
pub mod utils;

use crate::v7::endpoints::{errors::RpcError, Rpc, RpcEndpoints};

#[allow(clippy::too_many_arguments)]
pub async fn test_rpc_endpoints(
    url_list: Vec<Url>,
    sierra_path: &str,
    casm_path: &str,
    sierra_path_2: &str,
    casm_path_2: &str,
    class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<(), RpcError> {
    info!("{}", "⌛ Testing Rpc V7 endpoints -- START ⌛".yellow());

    let mut rpc = Rpc::new(get_random_url(url_list.clone())?)?;
    match rpc
        .add_declare_transaction_v2(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc add_declare_transaction V2 COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc add_declare_transaction V2 INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .add_declare_transaction_v3(
            sierra_path_2,
            casm_path_2,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc add_declare_transaction V3 COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc add_declare_transaction V3 INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .add_invoke_transaction_v1(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc add_invoke_transaction V1 COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc add_invoke_transaction V1 INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .add_invoke_transaction_v3(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc add_invoke_transaction V3 COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc add_invoke_transaction V3 INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .invoke_contract_v1(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc invoke_contract V1 COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc invoke_contract V1 INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .invoke_contract_v3(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc invoke_contract V3 COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc invoke_contract V3 INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.block_number().await {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc block_number COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc block_number INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.chain_id().await {
        Ok(_) => {
            info!("{} {}", "\n✓ Rpc chain_id COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc chain_id INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .call(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!("{} {}", "\n✓ Rpc call COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc call INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .estimate_message_fee(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc estimate_message_fee COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc estimate_message_fee INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.get_block_transaction_count().await {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_block_transaction_count COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_block_transaction_count INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.get_block_with_tx_hashes().await {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_block_with_tx_hashes COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_block_with_tx_hashes INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.get_block_with_txs().await {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_block_with_txs COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_block_with_txs INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.get_state_update().await {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_state_update COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_state_update INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.get_storage_at(erc20_eth_contract_address).await {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_storage_at COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_storage_at INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_transaction_status_succeeded(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_transaction_status_succeeded COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_transaction_status_succeeded INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_transaction_by_hash_invoke(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_transaction_by_hash_invoke COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_transaction_by_hash_invoke INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_transaction_by_hash_deploy_acc(
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_transaction_by_hash_deploy_acc COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_transaction_by_hash_deploy_acc INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_transaction_by_block_id_and_index(
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_transaction_by_block_id_and_index COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_transaction_by_block_id_and_index INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc.get_transaction_by_hash_non_existent_tx().await {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_transaction_by_hash_non_existent_tx COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_transaction_by_hash_non_existent_tx INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_transaction_receipt(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_transaction_receipt COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_transaction_receipt INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    // match rpc
    //     .get_transaction_receipt_revert(
    //         url.clone(),
    //         sierra_path,
    //         casm_path,
    //         class_hash,
    //         account_address,
    //         private_key,
    //         erc20_strk_contract_address,
    //         erc20_eth_contract_address,
    //         amount_per_test,
    //     )
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "\n✓ Rpc get_transaction_receipt_revert COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_transaction_receipt_revert INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_class(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!("{} {}", "\n✓ Rpc get_class COMPATIBLE".green(), "✓".green())
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_class INCOMPATIBLE:".red(),
            e.to_string().red(),
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_class_hash_at(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_class_hash_at COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {} {}",
            "✗ Rpc get_class_hash_at INCOMPATIBLE:".red(),
            e,
            "✗".red()
        ),
    }

    set_random_rpc_url(&mut rpc, url_list.clone())?;
    match rpc
        .get_class_at(
            sierra_path,
            casm_path,
            class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
    {
        Ok(_) => {
            info!(
                "{} {}",
                "\n✓ Rpc get_class_at COMPATIBLE".green(),
                "✓".green()
            )
        }
        Err(e) => error!(
            "{} {}",
            "✗ Rpc get_class_at INCOMPATIBLE:".red(),
            e.to_string().red(),
        ),
    }

    info!("{}", "🏁 Testing Devnet V7 endpoints -- END 🏁".yellow());

    Ok(())
}
