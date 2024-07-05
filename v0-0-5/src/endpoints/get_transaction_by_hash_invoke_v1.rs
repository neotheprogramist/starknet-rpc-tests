use crate::{
    call::Call,
    jsonrpc::{HttpTransport, JsonRpcClient},
    provider::{Provider, ProviderError},
    utilities::decalare_and_deploy,
    Account, AccountError,
};
use starknet_core::types::{Felt, InvokeTransactionV1};
use starknet_core::{
    types::{
        BlockId, BlockTag, BlockWithReceipts, InvokeTransaction, MaybePendingBlockWithReceipts,
        Transaction,
    },
    utils::get_selector_from_name,
};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GetTransactionByHashInvokeV1Error {
    #[error("Error getting response text")]
    ProviderError(#[from] ProviderError),

    #[error("Account error")]
    AccountError(String),

    #[error("Unexpected tx response type")]
    UnexecpectedTxResponseType(String),
}

pub async fn get_transaction_by_hash_invoke_v1(
    url: Url,
) -> Result<InvokeTransactionV1, GetTransactionByHashInvokeV1Error> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let (account, contract_address) = decalare_and_deploy(
        Felt::from_hex_unchecked(
            "0x4b3f4ba8c00a02b66142a4b1dd41a4dfab4f92650922a3280977b0f03c75ee1",
        ),
        Felt::from_hex_unchecked("0x57b2f8431c772e647712ae93cc616638"),
        Felt::from_hex_unchecked("0x534e5f5345504f4c4941"),
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
        url,
    )
    .await;
    let amount = Felt::from_hex("0x10").unwrap();
    let invoke_v1_result = account
        .execute_v1(vec![Call {
            to: contract_address,
            selector: get_selector_from_name("increase_balance").unwrap(),
            calldata: vec![amount],
        }])
        .send()
        .await
        .map_err(|err| GetTransactionByHashInvokeV1Error::AccountError(err.to_string()))?;

    let tx = rpc_client
        .get_transaction_by_hash(invoke_v1_result.transaction_hash)
        .await?;

    match tx {
        Transaction::Invoke(InvokeTransaction::V1(tx)) => Ok(tx),
        _ => Err(
            GetTransactionByHashInvokeV1Error::UnexecpectedTxResponseType(
                "Unexpected tx response type".to_string(),
            ),
        )?,
    }
}
