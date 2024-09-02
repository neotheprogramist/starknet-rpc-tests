use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_6_0::PriceUnit;

use crate::v6::rpc::signers::key_pair::SigningKey;

use super::create::AccountType;

#[allow(dead_code)]
fn serialize_as_decimal<S>(value: &Felt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let val = value;
    serializer.serialize_str(&format!("{val:#}"))
}

#[derive(Serialize)]
#[allow(dead_code)]
pub struct AccountCreateResponse {
    pub private_key: Felt,
    pub address: Felt,
    deployed: bool,
    #[serde(serialize_with = "crate::v6::rpc::accounts::creation::structs::serialize_as_decimal")]
    pub max_fee: Felt,
    pub message: String,
    pub class_hash: Felt,
    pub salt: Felt,
}

#[derive(Debug, Copy, Clone)]
pub struct GenerateAccountResponse {
    pub signing_key: SigningKey,
    pub address: Felt,
    pub deployed: bool,
    pub account_type: AccountType,
    pub class_hash: Felt,
    pub salt: Felt,
    pub max_fee: Felt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintRequest {
    pub amount: u128,
    pub address: Felt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintResponse {
    new_balance: String,
    unit: PriceUnit,
    tx_hash: Felt,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct MintResponse {
//     new_balance: U256,
//     unit: PriceUnit,
//     tx_hash: Felt,
// }
