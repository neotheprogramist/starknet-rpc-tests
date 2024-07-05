use crate::{
    call::Call,
    errors::errors::RunnerError,
    provider::{Provider, ProviderError},
    utilities::decalare_and_deploy,
    Account, ConnectedAccount,
};

use starknet_core::{
    types::{BlockId, BlockTag, Felt, FunctionCall},
    utils::{get_selector_from_name, NonAsciiNameError},
};

use starknet_types_core::felt::FromStrError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum InvokeError {
    #[error("Error getting response text")]
    CreateAccountError(String),

    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Error parsing hex string")]
    FromStrError(#[from] FromStrError),

    #[error("Runner error")]
    RunnerError(#[from] RunnerError),

    #[error("Unexpected receipt response type")]
    UnexpectedReceiptType,

    #[error("Unexpected execution result")]
    UnexpectedExecutionResult,

    #[error("Non ascii name error")]
    NonAsciiNameError(#[from] NonAsciiNameError),

    #[error("Account error")]
    AccountError(String),
}

pub async fn invoke(url: Url) -> Result<Vec<Felt>, InvokeError> {
    let (account, contract_address) = decalare_and_deploy(
        Felt::from_hex("0x4b3f4ba8c00a02b66142a4b1dd41a4dfab4f92650922a3280977b0f03c75ee1")?,
        Felt::from_hex("0x57b2f8431c772e647712ae93cc616638")?,
        Felt::from_hex("0x534e5f5345504f4c4941")?,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
        url.clone(),
    )
    .await;

    let amount = Felt::from_hex("0x10")?;

    account
        .execute_v3(vec![Call {
            to: contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![amount],
        }])
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
        .map_err(|err| InvokeError::AccountError(err.to_string()))?;

    let eth_balance = account
        .provider()
        .call(
            &FunctionCall {
                contract_address: contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;
    Ok(eth_balance)
}
