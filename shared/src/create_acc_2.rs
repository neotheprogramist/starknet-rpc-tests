use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use serde_json::json;
use starknet_accounts::{AccountDeployment, AccountFactory, OpenZeppelinAccountFactory};
use starknet_core::types::{BlockId, BlockTag};
use starknet_core::types::{FeeEstimate, FieldElement};
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_providers::Provider;
use starknet_signers::{LocalWallet, SigningKey};
use std::fmt;

#[allow(clippy::doc_markdown)]
#[derive(Clone, Debug)]
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

#[derive(Serialize)]
pub struct AccountCreateResponse {
    pub address: FieldElement,
    pub max_fee: FieldElement,
    pub add_profile: String,
    pub message: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    rpc_url: &str,
    account: &str,
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: FieldElement,
    account_type: AccountType,
    salt: Option<FieldElement>,
    add_profile: Option<String>,
    class_hash: Option<FieldElement>,
) -> Result<AccountCreateResponse, String> {
    let salt = extract_or_generate_salt(salt);

    let class_hash: FieldElement = FieldElement::from_hex_be(
        "0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
    )
    .unwrap();

    check_class_hash_exists(provider, class_hash).await?;

    let (account_json, max_fee) =
        generate_account(provider, salt, class_hash, &account_type).await?;

    // TODO: rm temporary resnpose
    Ok(AccountCreateResponse {
        address: FieldElement::from_dec_str("0x123").unwrap(),
        max_fee: FieldElement::from_dec_str("0x123").unwrap(),
        add_profile: "".to_string(),
        message: "".to_string(),
    })
}

pub fn extract_or_generate_salt(salt: Option<FieldElement>) -> FieldElement {
    salt.unwrap_or(FieldElement::from(OsRng.next_u64()))
}

pub async fn check_class_hash_exists(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: FieldElement,
) -> Result<(), String> {
    provider
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
        .map_err(|err| format!("Error checking class hash exists: {:?}", err))
        .map(|_| ())
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    class_hash: FieldElement,
    account_type: &AccountType,
) -> Result<(serde_json::Value, FieldElement), String> {
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

    Ok((account_json, fee_estimate.overall_fee))
}

pub async fn get_chain_id(provider: &JsonRpcClient<HttpTransport>) -> Result<FieldElement, String> {
    provider
        .chain_id()
        .await
        .map_err(|e| format!("Failed to fetch chain_id: {}", e))
}

async fn get_address_and_deployment_fee<T>(
    account_factory: T,
    salt: FieldElement,
) -> Result<(FieldElement, FeeEstimate), String>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy(salt);
    let fee = get_deployment_fee(&deployment)
        .await
        .map_err(|e| e.to_string())?;
    Ok((deployment.address(), fee))
}
async fn get_deployment_fee<'a, T>(
    account_deployment: &AccountDeployment<'a, T>,
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
    address: FieldElement,
    deployed: bool,
    legacy: bool,
    account_type: &AccountType,
    class_hash: Option<FieldElement>,
    salt: Option<FieldElement>,
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
