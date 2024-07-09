use crate::{
    errors::errors::RunnerError,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
    utilities::declare_contract_v3,
    ConnectedAccount, ExecutionEncoding, SingleOwnerAccount,
};
use colored::*;
use starknet_core::types::{BlockId, BlockTag, ContractClass, Felt, FlattenedSierraClass};
use starknet_signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use tracing::info;
use url::Url;
#[derive(Error, Debug)]
pub enum GetClassCairo1Error {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),

    #[error("Account error")]
    AccountError(String),

    #[error("Unexpected class type")]
    UnexpectedClassType,
}

pub async fn get_class_cairo_1(
    url: Url,
    chain_id: String,
) -> Result<FlattenedSierraClass, GetClassCairo1Error> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        "0xa20a02f0ac53692d144b20cb371a60d7",
    )?));

    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
        signer,
        Felt::from_hex("0x49dfb8ce986e21d354ac93ea65e6a11f639c1934ea253e5ff14ca62eca0f38e")?,
        Felt::from_hex(&chain_id)?,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let class_hash = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await?;

    let class = account
        .provider()
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await?;

    match class {
        ContractClass::Sierra(class) => {
            info!("{}", "Get Class Cairo 1 Compatible".green());
            Ok(class)
        }
        _ => Err(GetClassCairo1Error::UnexpectedClassType),
    }
}
