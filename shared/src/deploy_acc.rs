use serde::{Deserialize, Serialize};
use starknet_accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet_core::types::FieldElement;
use starknet_providers::{jsonrpc::HttpTransport, JsonRpcClient};
use starknet_signers::{LocalWallet, SigningKey};

use crate::create_acc::{AccountCreateResponse, AccountType};

// used in wait_for_tx. Txs will be fetched every 5s with timeout of 300s - so 60 attempts
#[allow(dead_code)]
pub const WAIT_TIMEOUT: u16 = 300;
#[allow(dead_code)]
pub const WAIT_RETRY_INTERVAL: u8 = 5;

pub struct Deploy {
    /// Name of the account to be deployed
    pub name: Option<String>,

    /// Max fee for the transaction
    pub max_fee: Option<FieldElement>,
}

#[derive(Clone, Copy)]
pub struct WaitForTx {
    pub wait: bool,
    pub wait_params: ValidatedWaitParams,
}

impl ValidatedWaitParams {
    pub fn new(retry_interval: u8, timeout: u16) -> Self {
        assert!(
            !(retry_interval == 0 || timeout == 0 || u16::from(retry_interval) > timeout),
            "Invalid values for retry_interval and/or timeout!"
        );

        Self {
            timeout,
            retry_interval,
        }
    }

    pub fn get_retries(&self) -> u16 {
        self.timeout / u16::from(self.retry_interval)
    }

    pub fn remaining_time(&self, steps_done: u16) -> u16 {
        steps_done * u16::from(self.retry_interval)
    }

    pub fn get_retry_interval(&self) -> u8 {
        self.retry_interval
    }

    pub fn get_timeout(&self) -> u16 {
        self.timeout
    }
}

impl Default for ValidatedWaitParams {
    fn default() -> Self {
        Self::new(WAIT_RETRY_INTERVAL, WAIT_TIMEOUT)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy, PartialEq)]
pub struct ValidatedWaitParams {
    timeout: u16,
    retry_interval: u8,
}

#[allow(unused_variables)]
pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    deploy_args: Deploy,
    chain_id: FieldElement,
    wait_conifg: WaitForTx,
    account_create_response: AccountCreateResponse,
) -> Result<InvokeResponse, String> {
    let private_key =
        SigningKey::from_secret_scalar(account_create_response.account_data.private_key);

    get_deployment_result(
        provider,
        account_create_response.account_data.account_type,
        account_create_response.account_data.class_hash,
        private_key,
        account_create_response.account_data.salt,
        chain_id,
        Some(account_create_response.max_fee),
        wait_conifg,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn get_deployment_result(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    class_hash: FieldElement,
    private_key: SigningKey,
    salt: FieldElement,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse, String> {
    match account_type {
        AccountType::Oz => {
            deploy_oz_account(
                provider,
                class_hash,
                private_key,
                salt,
                chain_id,
                max_fee,
                wait_config,
            )
            .await
        }
    }
}

async fn deploy_oz_account(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: FieldElement,
    private_key: SigningKey,
    salt: FieldElement,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse, String> {
    let factory = OpenZeppelinAccountFactory::new(
        class_hash,
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await
    .unwrap();

    deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
}

#[allow(unused_variables)]
async fn deploy_account<T>(
    account_factory: T,
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
    class_hash: FieldElement,
) -> Result<InvokeResponse, String>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy(salt);

    let deploy_max_fee = if let Some(max_fee) = max_fee {
        max_fee
    } else {
        match deployment.estimate_fee().await {
            Ok(max_fee) => max_fee.overall_fee,
            Err(_) => return Err("Failed to estimate fee".to_string()),
        }
    };
    let result = deployment.max_fee(deploy_max_fee).send().await;

    match result {
        Err(_) => Err("Deployment error occurred".to_string()),
        Ok(result) => {
            let return_value = InvokeResponse {
                transaction_hash: result.transaction_hash,
            };

            Ok(return_value)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InvokeResponse {
    pub transaction_hash: FieldElement,
}
