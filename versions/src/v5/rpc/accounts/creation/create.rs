use starknet_types_rpc::Felt;
use url::Url;

use crate::v5::rpc::{
    accounts::errors::CreationError,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
};

use super::{
    helpers::{check_class_hash_exists, extract_or_generate_salt, generate_account, OZ_CLASS_HASH},
    structs::{AccountCreateResponse, GenerateAccountResponse},
};

#[derive(Clone, Debug)]
pub enum AccountType {
    Oz,
}

pub async fn create(
    rpc_url: Url,
    account: &str,
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    account_type: AccountType,
    salt: Option<Felt>,
    class_hash: Option<Felt>,
) -> Result<GenerateAccountResponse, CreationError> {
    let salt = extract_or_generate_salt(salt);
    let class_hash = class_hash.unwrap_or_else(|| match account_type {
        AccountType::Oz => Felt::from_hex(OZ_CLASS_HASH).unwrap(),
    });
    check_class_hash_exists(provider, class_hash).await?;
    let account_response = generate_account(provider, salt, class_hash, &account_type).await?;
    tracing::info!("Account sucessfully created. Prefund address with at least {} tokens. It is good to send more in case of higher demand.", account_response.max_fee);
    Ok(account_response)
}
