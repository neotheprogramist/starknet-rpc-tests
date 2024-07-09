use crate::{
    account::create_mint_deploy::create_mint_deploy,
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::ProviderError,
    utilities::declare_contract_v3,
    ExecutionEncoding, SingleOwnerAccount,
};
use colored::*;
use starknet_core::types::{BlockId, BlockTag, Felt};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use tracing::info;
use url::Url;

#[derive(Error, Debug)]
pub enum AddDeclareTransactionError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),
}

pub async fn add_declare_transaction(
    url: Url,
    chain_id: String,
) -> Result<Felt, AddDeclareTransactionError> {
    let account_create_response = match create_mint_deploy(url.clone()).await {
        Ok(value) => value,
        Err(e) => return Err(AddDeclareTransactionError::CreateAccountError(e)),
    };

    let sender_address =
        Felt::from_hex("0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691")?;

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        Felt::from_hex("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
    ));

    let chain_id = Felt::from_hex("0x534e5f5345504f4c4941").unwrap();
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let response = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await?;
    info!("{}", "Add declare transaction compatible".green());
    Ok(response)
    // let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    // let account_create_response = match create_mint_deploy(url).await {
    //     Ok(value) => value,
    //     Err(e) => return Err(AddDeclareTransactionError::CreateAccountError(e)),
    // };
    // println!("{:?}", account_create_response.account_data);
    // let signer = LocalWallet::from(SigningKey::from_secret_scalar(
    //     account_create_response.account_data.private_key,
    // ));

    // let mut account = SingleOwnerAccount::new(
    //     rpc_client.clone(),
    //     signer,
    //     account_create_response.account_data.address,
    //     Felt::from_hex(&chain_id)?,
    //     ExecutionEncoding::New,
    // );
    // account.set_block_id(BlockId::Tag(BlockTag::Pending));
    // let response = declare_contract_v3(
    //     &account,
    //     "../target/dev/example_HelloStarknet.contract_class.json",
    //     "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    // )
    // .await?;
    // Ok(response)
}
