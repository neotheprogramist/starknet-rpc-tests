use rand::{rngs::OsRng, RngCore};
use starknet_crypto::FieldElement;
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
    chain_id: FieldElement,
    account_type: AccountType,
    salt: Option<FieldElement>,
    add_profile: Option<String>,
    class_hash: Option<FieldElement>,
) -> Result<(), String> {
    let salt = extract_or_generate_salt(salt);

    let OZ_CLASS_HASH: FieldElement = FieldElement::from_hex_be(
        "0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
    )
    .unwrap();

    let class_hash = class_hash.unwrap_or(match account_type {
        AccountType::Oz => OZ_CLASS_HASH,
    });
    check_class_hash_exists(provider, class_hash).await?;

    Ok(())
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
