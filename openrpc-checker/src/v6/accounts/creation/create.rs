use starknet_types_core::felt::Felt;

use crate::v6::{
    accounts::errors::CreationError,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
};

use super::{
    helpers::{extract_or_generate_salt, generate_account, OZ_CLASS_HASH},
    structs::GenerateAccountResponse,
};

#[derive(Clone, Copy, Debug)]
pub enum AccountType {
    Oz,
}

pub async fn create_account(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    salt: Option<Felt>,
    class_hash: Option<Felt>,
) -> Result<GenerateAccountResponse, CreationError> {
    let salt = extract_or_generate_salt(salt);
    let class_hash = class_hash.unwrap_or_else(|| match account_type {
        AccountType::Oz => Felt::from_hex(OZ_CLASS_HASH).unwrap(),
    });
    let account_response = generate_account(provider, salt, class_hash, &account_type).await?;
    Ok(account_response)
}
