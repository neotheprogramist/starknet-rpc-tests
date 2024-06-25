use rand::{rngs::OsRng, RngCore};
use starknet_accounts::{AccountDeploymentV1, AccountFactory, OpenZeppelinAccountFactory};
use starknet_core::types::{FeeEstimate, Felt};
use starknet_crypto::FieldElement;

use starknet_signers::{LocalWallet, SigningKey};
use utils::{
    codegen::BlockTag,
    models::BlockId,
    provider::Provider,
    transports::{http::HttpTransport, JsonRpcClient},
};

enum AccountType {
    Oz,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    rpc_url: &str,
    account: &str,
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    account_type: AccountType,
    salt: Option<FieldElement>,
    add_profile: Option<String>,
    class_hash: Option<FieldElement>,
) -> Result<(), String> {
    let salt = extract_or_generate_salt(salt);

    let ozz_class_hash: FieldElement = FieldElement::from_hex_be(
        "0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
    )
    .unwrap();

    let class_hash = class_hash.unwrap_or(match account_type {
        AccountType::Oz => ozz_class_hash,
    });
    check_class_hash_exists(provider, class_hash).await?;
    generate_account(provider, salt, class_hash, &account_type).await;

    Ok(())
}

pub async fn check_if_legacy_contract(
    class_hash: Option<FieldElement>,
    address: FieldElement,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<bool> {
    let contract_class = match class_hash {
        Some(class_hash) => provider.get_class(BlockId::Tag(Pending), class_hash).await,
        None => provider.get_class_at(BlockId::Tag(Pending), address).await,
    }
    .map_err(handle_rpc_error)?;

    Ok(is_legacy_contract(&contract_class))
}

pub fn field_element_to_felt(fe: FieldElement) -> Felt {
    let bytes: [u8; 32] = fe.to_bytes_be();
    Felt::from_bytes_be(&bytes)
}

pub fn felt_to_field_element(felt: Felt) -> FieldElement {
    let bytes = felt.to_bytes_be();
    FieldElement::from_bytes_be(&bytes).unwrap()
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    class_hash: FieldElement,
    account_type: &AccountType,
) -> Result<(), String> {
    let class_hash = field_element_to_felt(class_hash);
    let chain_id = field_element_to_felt(get_chain_id(provider).await?);
    let key = SigningKey::from_random();
    let signer = LocalWallet::from_signing_key(key.clone());

    let (address, fee_estimate) = match account_type {
        AccountType::Oz => {
            // let factory =
            OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
            get_address_and_deployment_fee(factory, salt).await?
        }
    };

    let legacy = check_if_legacy_contract(Some(class_hash), address, provider).await?;

    Ok(())
}

async fn get_address_and_deployment_fee<T>(
    account_factory: T,
    salt: FieldElement,
) -> Result<(FieldElement, FeeEstimate), String>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy_v1(field_element_to_felt(salt));
    let fee_estimate = get_deployment_fee(&deployment)
        .await
        .map_err(|e| format!("Failed to get deployment fee: {}", e))?;
    Ok((felt_to_field_element(deployment.address()), fee_estimate))
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

pub async fn get_chain_id(provider: &JsonRpcClient<HttpTransport>) -> Result<FieldElement, String> {
    provider
        .chain_id()
        .await
        .map_err(|err| format!("Failed to get chain id: {}", err))
}

pub fn extract_or_generate_salt(salt: Option<FieldElement>) -> FieldElement {
    match salt {
        Some(s) => s,
        None => FieldElement::from(OsRng.next_u64()),
    }
}

pub async fn check_class_hash_exists(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: FieldElement,
) -> Result<(), String> {
    match provider
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Class hash {} not found: {}", class_hash, e)),
    }
}
