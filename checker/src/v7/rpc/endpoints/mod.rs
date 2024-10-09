pub mod declare_contract;
pub mod deploy_contract;
pub mod endpoints_functions;
pub mod errors;
pub mod utils;

use colored::*;
use endpoints_functions::{
    add_declare_transaction_v2, add_declare_transaction_v3, add_invoke_transaction_v1,
    add_invoke_transaction_v3, block_number, call, chain_id, estimate_message_fee,
    get_block_transaction_count, get_block_with_tx_hashes, get_block_with_txs, get_class,
    get_class_at, get_class_hash_at, get_state_update, get_storage_at,
    get_transaction_by_block_id_and_index, get_transaction_by_hash_deploy_acc,
    get_transaction_by_hash_invoke, get_transaction_by_hash_non_existent_tx,
    get_transaction_receipt, get_transaction_status_succeeded, invoke_contract_v1,
    invoke_contract_v3,
};
use errors::RpcError;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::{
    AddInvokeTransactionResult, BlockWithTxHashes, BlockWithTxs, ContractClass, DeployAccountTxnV3,
    DeployTxnReceipt, FeeEstimate, InvokeTxnV1, StateUpdate, TxnStatus,
};

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

#[allow(dead_code)]
pub trait RpcEndpoints {
    fn add_declare_transaction_v2(
        &self,
        sierra_path: &str,
        casm_path: &str,
        account_class_hash: Option<Felt>,
        account_address: Option<Felt>,
        private_key: Option<Felt>,
        erc20_strk_contract_address: Option<Felt>,
        erc20_eth_contract_address: Option<Felt>,
        amount_per_test: Option<Felt>,
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

    async fn invoke_contract_v1(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<AddInvokeTransactionResult<Felt>, RpcError>;

    async fn invoke_contract_v3(
        &self,
        url: Url,
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
        account_class_hash: Option<Felt>,
        account_address: Option<Felt>,
        private_key: Option<Felt>,
        erc20_strk_contract_address: Option<Felt>,
        erc20_eth_contract_address: Option<Felt>,
        amount_per_test: Option<Felt>,
    ) -> Result<Felt, RpcError> {
        add_declare_transaction_v2(
            self.url.clone(),
            sierra_path,
            casm_path,
            account_class_hash,
            account_address,
            private_key,
            erc20_strk_contract_address,
            erc20_eth_contract_address,
            amount_per_test,
        )
        .await
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

    async fn invoke_contract_v1(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
        invoke_contract_v1(url.clone(), sierra_path, casm_path).await
    }

    async fn invoke_contract_v3(
        &self,
        url: Url,
        sierra_path: &str,
        casm_path: &str,
    ) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
        invoke_contract_v3(url.clone(), sierra_path, casm_path).await
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
        get_transaction_by_hash_invoke(url.clone(), sierra_path, casm_path).await
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
        get_transaction_receipt(url.clone(), sierra_path, casm_path).await
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
    class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<(), RpcError> {
    info!("{}", "⌛ Testing Rpc V7 endpoints -- START ⌛".yellow());

    let rpc = Rpc::new(url.clone())?;
    restart_devnet(url.clone()).await?;

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
                "✓ Rpc add_declare_transaction V2 COMPATIBLE".green(),
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
    // restart_devnet(url.clone()).await?;

    // match rpc.add_declare_transaction_v3(sierra_path, casm_path).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc add_declare_transaction V3 COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc add_declare_transaction V3 INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }
    // restart_devnet(url.clone()).await?;

    // match rpc.add_invoke_transaction_v1(sierra_path, casm_path).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc add_invoke_transaction V1 COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc add_invoke_transaction V1 INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }
    // restart_devnet(url.clone()).await?;

    // match rpc.add_invoke_transaction_v3(sierra_path, casm_path).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc add_invoke_transaction V3 COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc add_invoke_transaction V3 INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // restart_devnet(url.clone()).await?;

    // match rpc
    //     .invoke_contract_v1(url.clone(), sierra_path, casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc invoke_contract V1 COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc invoke_contract V1 INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // restart_devnet(url.clone()).await?;

    // match rpc
    //     .invoke_contract_v3(url.clone(), sierra_path, casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc invoke_contract V3 COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc invoke_contract V3 INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

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
    //         "✗ Rpc block_number INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

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

    // restart_devnet(url.clone()).await?;

    // match rpc
    //     .estimate_message_fee(url.clone(), sierra_path, casm_path)
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
    //         "✗".red()
    //     ),
    // }
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
    // match rpc.get_block_with_tx_hashes(url.clone()).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_block_with_tx_hashes COMPATIBLE".green(),
    //             "✓".green()
    //         )
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

    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_transaction_status_succeeded(url.clone(), sierra_path, casm_path)
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
    //     .get_transaction_by_hash_invoke(url.clone(), sierra_path, casm_path)
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
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_transaction_by_block_id_and_index INCOMPATIBLE:".red(),
    //         e.to_string().red(),
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

    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_transaction_receipt(url.clone(), sierra_path, casm_path)
    //     .await
    // {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_transaction_receipt COMPATIBLE".green(),
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
    // match rpc.get_class(url.clone(), sierra_path, casm_path).await {
    //     Ok(_) => {
    //         info!("{} {}", "✓ Rpc get_class COMPATIBLE".green(), "✓".green())
    //     }
    //     Err(e) => error!(
    //         "{} {} {}",
    //         "✗ Rpc get_class INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //         "✗".red()
    //     ),
    // }

    // restart_devnet(url.clone()).await?;
    // match rpc
    //     .get_class_hash_at(url.clone(), sierra_path, casm_path)
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
    //         "{} {} {}",
    //         "✗ Rpc get_class_hash_at INCOMPATIBLE:".red(),
    //         e,
    //         "✗".red()
    //     ),
    // }
    // restart_devnet(url.clone()).await?;

    // match rpc.get_class_at(url.clone(), sierra_path, casm_path).await {
    //     Ok(_) => {
    //         info!(
    //             "{} {}",
    //             "✓ Rpc get_class_at COMPATIBLE".green(),
    //             "✓".green()
    //         )
    //     }
    //     Err(e) => error!(
    //         "{} {}",
    //         "✗ Rpc get_class_at INCOMPATIBLE:".red(),
    //         e.to_string().red(),
    //     ),
    // }

    info!("{}", "🏁 Testing Devnet V7 endpoints -- END 🏁".yellow());

    Ok(())
}
