use starknet_types_rpc::Felt;
use tracing::info;

use crate::v5::rpc::{
    accounts::errors::CreationError,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
};

use super::{
    helpers::{extract_or_generate_salt, generate_account, OZ_CLASS_HASH},
    structs::GenerateAccountResponse,
};

#[derive(Clone, Debug)]
pub enum AccountType {
    Oz,
}

pub async fn create(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    salt: Option<Felt>,
    class_hash: Option<Felt>,
) -> Result<GenerateAccountResponse, CreationError> {
    let salt = extract_or_generate_salt(salt);
    info!("SALT: {}", salt);
    let class_hash = class_hash.unwrap_or_else(|| match account_type {
        AccountType::Oz => Felt::from_hex(OZ_CLASS_HASH).unwrap(),
    });
    info!("CLASS HASH {}", class_hash);
    // check_class_hash_exists(provider, class_hash).await?;
    info!("STARTING TO GENERATE ACCOUNT");
    let account_response = generate_account(provider, salt, class_hash, &account_type).await?;
    tracing::info!("Account sucessfully created. Prefund address with at least {} tokens. It is good to send more in case of higher demand.", account_response.max_fee);
    Ok(account_response)
}

// pub async fn mint(base_url: Url, mint_request: &MintRequest) -> Result<MintResponse, MintError> {
//     let mint_url = match base_url.join("mint") {
//         Ok(url) => url,
//         Err(e) => return Err(MintError::JoinUrlError(e)),
//     };

//     let response = Client::new()
//         .post(mint_url)
//         .header("Content-type", "application/json")
//         .json(mint_request)
//         .send()
//         .await?;

//     if !response.status().is_success() {
//         let status_code = response.status();
//         let error_message = response
//             .text()
//             .await
//             .map_err(|_| MintError::ResponseTextError)?;
//         Err(MintError::ResponseStatusError {
//             status_code,
//             message: Some(error_message),
//         })
//     } else {
//         let mint_response = response
//             .json::<MintResponse>()
//             .await
//             .map_err(|_| MintError::ResponseParseError)?;
//         Ok(mint_response)
//     }
// }
