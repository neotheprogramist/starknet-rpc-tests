use colored::Colorize;
use rand::rngs::OsRng;
use rand::RngCore;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
// use starknet_accounts::{AccountDeploymentV1, AccountFactory, OpenZeppelinAccountFactory};
use starknet_core::types::{BlockId, BlockTag, FeeEstimate, Felt};

use crate::factory::open_zeppelin::OpenZeppelinAccountFactory;
use crate::factory::{AccountDeploymentV1, AccountFactory};
use starknet_signers::{LocalWallet, SigningKey};
use std::fmt;
use tracing::info;

use crate::jsonrpc::{HttpTransport, JsonRpcClient};
use crate::provider::Provider;

#[allow(clippy::doc_markdown)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AccountType {
    /// OpenZeppelin account implementation
    Oz,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AccountType::Oz => write!(f, "open_zeppelin"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountData {
    pub address: Felt,
    pub class_hash: Felt,
    pub deployed: bool,
    pub legacy: bool,
    pub private_key: Felt,
    pub public_key: Felt,
    pub salt: Felt,
    #[serde(rename = "type")]
    pub account_type: AccountType,
}

#[derive(Serialize, Debug, Clone)]
pub struct AccountCreateResponse {
    pub account_json: serde_json::Value,
    pub account_data: AccountData,
    pub max_fee: Felt,
    pub add_profile: String,
    pub message: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    salt: Option<Felt>,
) -> Result<AccountCreateResponse, String> {
    let salt = extract_or_generate_salt(salt);

    // env
    let class_hash: Felt =
        Felt::from_hex("0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f")
            .unwrap();

    check_class_hash_exists(provider, class_hash).await?;

    let (account_json, account_data, max_fee) =
        generate_account(provider, salt, class_hash, &account_type).await?;

    // TODO: REDUNDANT / change name
    match mint_tokens(
        3010000000000000,
        account_json["address"].as_str().unwrap().to_string(),
    )
    .await
    {
        Ok(_) => {
            println!("Tokens minted successfully")
        }
        Err(e) => {
            return Err(format!(
                "Failed to mint tokens for account: {}. Reason: {}",
                account_json["address"].as_str().unwrap(),
                e
            ))
        }
    };
    Ok(AccountCreateResponse {
        account_json,
        account_data,
        max_fee,
        add_profile: "No profile added to snfoundry.toml".to_string(),
        message: "Account successfully created. Prefund generated address with at least <max_fee> tokens. It is good to send more in the case of higher demand.".to_string(),
    })
}

pub fn extract_or_generate_salt(salt: Option<Felt>) -> Felt {
    salt.unwrap_or(Felt::from(OsRng.next_u64()))
}

pub async fn check_class_hash_exists(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: Felt,
) -> Result<(), String> {
    provider
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
        .map_err(|err| format!("Error checking class hash exists: {:?}", err))
        .map(|_| ())
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: Felt,
    class_hash: Felt,
    account_type: &AccountType,
) -> Result<(serde_json::Value, AccountData, Felt), String> {
    let chain_id = get_chain_id(provider).await?;
    let private_key = SigningKey::from_random();
    let signer = LocalWallet::from_signing_key(private_key.clone());

    let (address, fee_estimate) = match account_type {
        AccountType::Oz => {
            let factory = OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider)
                .await
                .map_err(|e| e.to_string())?;
            get_address_and_deployment_fee(factory, salt).await?
        }
    };

    let account_json = prepare_account_json(
        &private_key,
        address,
        false,
        false,
        account_type,
        Some(class_hash),
        Some(salt),
    );

    let account_struct = AccountData {
        address,
        class_hash,
        deployed: false,
        legacy: false,
        private_key: private_key.secret_scalar(),
        public_key: private_key.verifying_key().scalar(),
        salt,
        account_type: account_type.clone(),
    };

    Ok((account_json, account_struct, fee_estimate.overall_fee))
}

pub async fn get_chain_id(provider: &JsonRpcClient<HttpTransport>) -> Result<Felt, String> {
    provider
        .chain_id()
        .await
        .map_err(|e| format!("Failed to fetch chain_id: {}", e))
}

async fn get_address_and_deployment_fee<T>(
    account_factory: T,
    salt: Felt,
) -> Result<(Felt, FeeEstimate), String>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy_v1(salt);
    let fee = get_deployment_fee(&deployment)
        .await
        .map_err(|e| e.to_string())?;
    Ok((deployment.address(), fee))
}
async fn get_deployment_fee<'a, T>(
    account_deployment: &AccountDeploymentV1<'a, T>,
) -> Result<FeeEstimate, String>
where
    T: AccountFactory + Sync,
{
    let fee_estimate = account_deployment.estimate_fee().await;

    match fee_estimate {
        Ok(fee_estimate) => Ok(fee_estimate),
        Err(err) => Err(format!(
            "Failed to estimate account deployment fee. Reason: {}",
            err
        )),
    }
}

pub fn prepare_account_json(
    private_key: &SigningKey,
    address: Felt,
    deployed: bool,
    legacy: bool,
    account_type: &AccountType,
    class_hash: Option<Felt>,
    salt: Option<Felt>,
) -> serde_json::Value {
    let mut account_json = json!({
        "private_key": format!("{:#x}", private_key.secret_scalar()),
        "public_key": format!("{:#x}", private_key.verifying_key().scalar()),
        "address": format!("{address:#x}"),
        "type": format!("{account_type}"),
        "deployed": deployed,
        "legacy": legacy,
    });

    if let Some(salt) = salt {
        account_json["salt"] = serde_json::Value::String(format!("{salt:#x}"));
    }
    if let Some(class_hash) = class_hash {
        account_json["class_hash"] = serde_json::Value::String(format!("{class_hash:#x}"));
    }

    account_json
}

#[derive(Serialize, Deserialize, Debug)]
struct MintRequest {
    amount: u128,
    address: String,
}

// serialize reposnse
pub async fn mint_tokens(amount: u128, address: String) -> Result<(), reqwest::Error> {
    let response = Client::new()
        .post("http://127.0.0.1:5050/mint")
        .header("Content-type", "application/json")
        .json(&MintRequest { amount, address })
        .send()
        .await?;

    if response.status().is_success() {
        info!("{}", "Token minting successful".green());
    } else {
        info!(
            "Token minting failed with status: {}",
            response.status().to_string().red()
        );
    }

    Ok(())
}
