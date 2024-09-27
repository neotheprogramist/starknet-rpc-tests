use std::str::FromStr;

use super::endpoints::postman_load;
use super::models::PostmanLoadL1MessagingContractParams;
use crate::v5::devnet::errors::DevnetError;
use crate::v5::devnet::models::MsgToL2;
use crate::v5::rpc::endpoints::endpoints_functions::get_transaction_receipt;
use crate::v5::rpc::endpoints::utils::get_selector_from_name;
use crate::v5::rpc::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use crate::v5::rpc::providers::provider::Provider;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_5_0::{BlockId, BlockTag, MsgFromL1};
use url::Url;

pub async fn prepare_postman_send_message_to_l2(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    l1_url: Url,
) -> Result<MsgToL2, DevnetError> {
    let receipt = get_transaction_receipt(url.clone(), sierra_path, casm_path).await?;

    let estimate = JsonRpcClient::new(HttpTransport::new(url.clone()))
        .estimate_message_fee(
            MsgFromL1 {
                from_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                to_address: receipt.contract_address,
                entry_point_selector: get_selector_from_name("deposit")?,
                payload: vec![(1_u32).into(), (10_u32).into()],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;

    let l1_contract_address = postman_load(
        url,
        PostmanLoadL1MessagingContractParams {
            network_url: l1_url.to_string(),
            address: Some("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()),
        },
    )
    .await?;

    let msg_to_l2 = MsgToL2 {
        l2_contract_address: receipt.contract_address,
        entry_point_selector: get_selector_from_name("deposit")?,
        l1_contract_address: Felt::from_str(&l1_contract_address.messaging_contract_address)?,
        payload: vec![(1_u32).into(), (10_u32).into()],
        paid_fee_on_l1: estimate.overall_fee.into(),
        nonce: Felt::ZERO,
    };
    Ok(msg_to_l2)
}
