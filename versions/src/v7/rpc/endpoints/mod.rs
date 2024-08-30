pub mod declare_contract;
pub mod deploy_contract;
pub mod endpoints;
pub mod errors;
pub mod utils;

use colored::*;
use endpoints::{
    add_declare_transaction_v2, add_declare_transaction_v3, add_invoke_transaction_v1,
    add_invoke_transaction_v3, block_number, call, chain_id, estimate_message_fee,
    get_block_transaction_count, get_block_with_tx_hashes, get_block_with_txs, get_class,
    get_class_at, get_class_hash_at, get_state_update, get_storage_at,
    get_transaction_by_block_id_and_index, get_transaction_by_hash_deploy_acc,
    get_transaction_by_hash_invoke, get_transaction_by_hash_non_existent_tx,
    get_transaction_receipt, get_transaction_status_succeeded,
};
use errors::RpcError;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{v0_7_1::{
    AddInvokeTransactionResult, BlockWithTxHashes, BlockWithTxs, ContractClass, DeployAccountTxnV1, DeployAccountTxnV3,
    DeployTxnReceipt, FeeEstimate, InvokeTxnV1, StateUpdate, TxnStatus,
}};

use tracing::{error, info};
use url::Url;
use utils::restart_devnet;

pub struct Rpc {
    pub url: Url,
}

impl Rpc {
    pub fn new(url: Url) -> Result<Self, RpcError> {
        Ok(Self { url })
    }
}

pub trait RpcEndpoints {
    fn add_declare_transaction_v2(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> impl std::future::Future<Output = Result<Felt, RpcError>> + Send;

    fn add_declare_transaction_v3(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> impl std::future::Future<Output = Result<Felt, RpcError>> + Send;

    async fn add_invoke_transaction_v1(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<AddInvokeTransactionResult<Felt>, RpcError>;

    async fn add_invoke_transaction_v3(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<AddInvokeTransactionResult<Felt>, RpcError>;

    async fn block_number(&self, url: Url) -> Result<u64, RpcError>;

    async fn chain_id(&self, url: Url) -> Result<Felt, RpcError>;

    async fn call(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<Vec<Felt>, RpcError>;

    async fn estimate_message_fee(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<FeeEstimate<Felt>, RpcError>;

    async fn get_block_transaction_count(&self, url: Url) -> Result<u64, RpcError>;

    async fn get_block_with_tx_hashes(&self, url: Url)
        -> Result<BlockWithTxHashes<Felt>, RpcError>;

    async fn get_block_with_txs(&self, url: Url) -> Result<BlockWithTxs<Felt>, RpcError>;

    async fn get_state_update(&self, url: Url) -> Result<StateUpdate<Felt>, RpcError>;

    async fn get_storage_at(&self, url: Url) -> Result<Felt, RpcError>;

    async fn get_transaction_status_succeeded(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<TxnStatus, RpcError>;

    async fn get_transaction_by_hash_invoke(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<InvokeTxnV1<Felt>, RpcError>;

    async fn get_transaction_by_hash_deploy_acc(
        &self,
        url: Url,
    ) -> Result<DeployAccountTxnV3<Felt>, RpcError>;

    async fn get_transaction_by_block_id_and_index(
        &self,
        url: Url,
    ) -> Result<InvokeTxnV1<Felt>, RpcError>;

    async fn get_transaction_by_hash_non_existent_tx(&self, url: Url) -> Result<(), RpcError>;

    async fn get_transaction_receipt(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<DeployTxnReceipt<Felt>, RpcError>;

    async fn get_class(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<ContractClass<Felt>, RpcError>;

    async fn get_class_hash_at(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<Felt, RpcError>;

    async fn get_class_at(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<ContractClass<Felt>, RpcError>;
}

impl RpcEndpoints for Rpc {
    async fn add_declare_transaction_v2(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<Felt, RpcError> {
        add_declare_transaction_v2(self.url.clone(), sierra_path, casm_path).await
    }

    async fn add_declare_transaction_v3(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<Felt, RpcError> {
        add_declare_transaction_v3(self.url.clone(), sierra_path, casm_path).await
    }

    async fn add_invoke_transaction_v1(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
        add_invoke_transaction_v1(self.url.clone(), sierra_path, casm_path).await
    }

    async fn add_invoke_transaction_v3(
        &self,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
        add_invoke_transaction_v3(self.url.clone(), sierra_path, casm_path).await
    }

    async fn block_number(&self, url: Url) -> Result<u64, RpcError> {
        block_number(url.clone()).await
    }

    async fn chain_id(&self, url: Url) -> Result<Felt, RpcError> {
        chain_id(url.clone()).await
    }

    async fn call(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<Vec<Felt>, RpcError> {
        call(url.clone(), sierra_path, casm_path).await
    }

    async fn estimate_message_fee(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<FeeEstimate<Felt>, RpcError> {
        estimate_message_fee(url.clone(), sierra_path, casm_path).await
    }

    async fn get_block_transaction_count(&self, url: Url) -> Result<u64, RpcError> {
        get_block_transaction_count(url.clone()).await
    }

    async fn get_block_with_tx_hashes(
        &self,
        url: Url,
    ) -> Result<BlockWithTxHashes<Felt>, RpcError> {
        get_block_with_tx_hashes(url.clone()).await
    }

    async fn get_block_with_txs(&self, url: Url) -> Result<BlockWithTxs<Felt>, RpcError> {
        get_block_with_txs(url.clone()).await
    }

    async fn get_state_update(&self, url: Url) -> Result<StateUpdate<Felt>, RpcError> {
        get_state_update(url.clone()).await
    }

    async fn get_storage_at(&self, url: Url) -> Result<Felt, RpcError> {
        get_storage_at(url.clone()).await
    }

    async fn get_transaction_status_succeeded(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<TxnStatus, RpcError> {
        get_transaction_status_succeeded(url.clone(), sierra_path, casm_path).await
    }

    async fn get_transaction_by_hash_invoke(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<InvokeTxnV1<Felt>, RpcError> {
        get_transaction_by_hash_invoke(url.clone(), &sierra_path, &casm_path).await
    }

    async fn get_transaction_by_hash_deploy_acc(
        &self,
        url: Url,
    ) -> Result<DeployAccountTxnV3<Felt>, RpcError> {
        get_transaction_by_hash_deploy_acc(url.clone()).await
    }

    async fn get_transaction_by_block_id_and_index(
        &self,
        url: Url,
    ) -> Result<InvokeTxnV1<Felt>, RpcError> {
        get_transaction_by_block_id_and_index(url.clone()).await
    }

    async fn get_transaction_by_hash_non_existent_tx(&self, url: Url) -> Result<(), RpcError> {
        get_transaction_by_hash_non_existent_tx(url.clone()).await
    }

    async fn get_transaction_receipt(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<DeployTxnReceipt<Felt>, RpcError> {
        get_transaction_receipt(url.clone(), &sierra_path, &casm_path).await
    }

    async fn get_class(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<ContractClass<Felt>, RpcError> {
        get_class(url.clone(), sierra_path, casm_path).await
    }

    async fn get_class_hash_at(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<Felt, RpcError> {
        get_class_hash_at(url.clone(), sierra_path, casm_path).await
    }

    async fn get_class_at(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<ContractClass<Felt>, RpcError> {
        get_class_at(url.clone(), sierra_path, casm_path).await
    }
}

pub async fn test_rpc_endpoints_v0_0_7(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
) -> Result<(), RpcError> {
    info!("{}", "⌛ Testing Rpc V7 endpoints -- START ⌛".yellow());

    let rpc = Rpc::new(url.clone())?;
    restart_devnet(url.clone()).await?;
    match rpc.add_declare_transaction_v3(&sierra_path, &casm_path).await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Rpc add_declare_transaction V3 COMPATIBLE".green(),
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
    restart_devnet(url.clone()).await?;
    match rpc.add_invoke_transaction_v1(&sierra_path, &casm_path).await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Rpc add_invoke_transaction V1 COMPATIBLE".green(),
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
    restart_devnet(url.clone()).await?;
    match rpc.add_invoke_transaction_v3(&sierra_path, &casm_path).await {
        Ok(_) => {
            info!(
                "{} {}",
                "✓ Rpc add_invoke_transaction V3 COMPATIBLE".green(),
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


    // match rpc.block_number(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc block_number COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         e.to_string().red(),

    // match rpc.chain_id(url.clone()).await {
    //     Ok(_) => {
    //         info!("{} {}", "✓ Rpc chain_id COMPATIBLE".green(), "✓".green())
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc chain_id INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // NOT WORKING
    // restart_devnet(url.clone()).await?;
    // match rpc.call(url.clone(), sierra_path, casm_path).await {
    //     Ok(_) => {
    //         info!("{} {}", "✓ Rpc call COMPATIBLE".green(), "✓".green())
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc call INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // NOT WORKING
    // restart_devnet(url.clone()).await?;

    // match rpc
    //     .estimate_message_fee(url.clone(), &sierra_path, &casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc estimate_message_fee COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc estimate_message_fee INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //     ),
    //  }
    // match rpc.get_block_transaction_count(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_block_transaction_count COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_block_transaction_count INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    //     Ok(_) => {
    //         info!(
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_block_with_tx_hashes INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // match rpc.get_block_with_txs(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_block_with_txs COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_block_with_txs INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // match rpc.get_state_update(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_state_update COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_state_update INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // match rpc.get_storage_at(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_storage_at COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_storage_at INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // NOT WORKING
    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_transaction_status_succeeded(url.clone(), &sierra_path, &casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_transaction_status_succeeded COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_transaction_status_succeeded INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_transaction_by_hash_invoke(url.clone(), &sierra_path, &casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_transaction_by_hash_invoke COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_transaction_by_hash_invoke INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // match rpc.get_transaction_by_hash_deploy_acc(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_transaction_by_hash_deploy_acc COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_transaction_by_hash_deploy_acc INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // match rpc.get_transaction_by_block_id_and_index(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_transaction_by_block_id_and_index COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //         "✗".red()
    //     ),
    // }

    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_transaction_by_hash_non_existent_tx(url.clone())
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_transaction_by_hash_non_existent_tx COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_transaction_by_hash_non_existent_tx INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    //  NOT WORKING
    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_transaction_receipt(url.clone(), &sierra_path, &casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_transaction_receiptCOMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_transaction_receipt INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // restart_devnet(url.clone()).await?;
    // match rpc.get_class(url.clone(), &sierra_path, &casm_path).await {
    //     Ok(_) => {
    //     }
    //     Err(e) => error!(
    //     ),
    // }

    //  NOT WORKING
    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_class_hash_at(url.clone(), &sierra_path, &casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_class_hash_at COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {}",
    //         "✗ Rpc get_class_hash_at INCOMPATIBLE:".red(),
    //         "✗".red())
    // }
    // NOT WORKING
    // restart_devnet(url.clone()).await?;

    // match rpc
    //     .get_class_at(url.clone(), &sierra_path, &casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_class_at COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_class_at INCOMPATIBLE:".red(),
    //         e.to_string().red(),)
    // }

    info!("{}", "🏁 Testing Devnet V7 endpoints -- END 🏁".yellow());

    Ok(())
}
