use std::sync::Arc;

use super::codegen::{DataAvailabilityMode, EntryPointsByType, ResourceBoundsMapping};
use crate::utils::byte_array::base64::serialize as base64_ser;
use crate::utils::serde_impls::u64_hex;
use crate::utils::unsigned_field_element::UfeHex;
use ::serde::Deserialize;
use serde::{Serialize, Serializer};
use serde_with::serde_as;
use starknet_crypto::FieldElement;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionRequest {
    Declare(DeclareTransaction),
    // InvokeFunction(InvokeFunctionTransaction),
    // DeployAccount(DeployAccountTransaction),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum DeclareTransaction {
    // V1(DeclareV1Transaction),
    // V2(DeclareV2Transaction),
    V3(DeclareV3Transaction),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct CompressedSierraClass {
    #[serde(serialize_with = "base64_ser")]
    pub sierra_program: Vec<u8>,
    pub contract_class_version: String,
    pub entry_points_by_type: EntryPointsByType,
    pub abi: String,
}

#[derive(Debug)]
pub struct DeclareV3Transaction {
    pub contract_class: Arc<CompressedSierraClass>,
    /// Hash of the compiled class obtained by running `starknet-sierra-compile` on the Sierra
    /// class. This is required because at the moment, Sierra compilation is not proven, allowing
    /// the sequencer to run arbitrary code if this is not signed. It's expected that in the future
    /// this will no longer be required.
    pub compiled_class_hash: FieldElement,
    /// The address of the account contract sending the declaration transaction.
    pub sender_address: FieldElement,
    /// Additional information given by the caller that represents the signature of the transaction.
    pub signature: Vec<FieldElement>,
    /// A sequential integer used to distinguish between transactions and order them.
    pub nonce: FieldElement,
    pub nonce_data_availability_mode: DataAvailabilityMode,
    pub fee_data_availability_mode: DataAvailabilityMode,
    pub resource_bounds: ResourceBoundsMapping,
    pub tip: u64,
    pub paymaster_data: Vec<FieldElement>,
    pub account_deployment_data: Vec<FieldElement>,
    pub is_query: bool,
}
const QUERY_VERSION_THREE: FieldElement = FieldElement::from_mont([
    18446744073700081569,
    17407,
    18446744073709551584,
    576460752142432688,
]);

impl Serialize for DeclareV3Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[serde_as]
        #[derive(Serialize)]
        struct Versioned<'a> {
            #[serde_as(as = "UfeHex")]
            version: FieldElement,
            contract_class: &'a CompressedSierraClass,
            #[serde_as(as = "UfeHex")]
            compiled_class_hash: &'a FieldElement,
            #[serde_as(as = "UfeHex")]
            sender_address: &'a FieldElement,
            #[serde_as(as = "Vec<UfeHex>")]
            signature: &'a Vec<FieldElement>,
            #[serde_as(as = "UfeHex")]
            nonce: &'a FieldElement,
            nonce_data_availability_mode: &'a DataAvailabilityMode,
            fee_data_availability_mode: &'a DataAvailabilityMode,
            resource_bounds: &'a ResourceBoundsMapping,
            #[serde(with = "u64_hex")]
            tip: &'a u64,
            #[serde_as(as = "Vec<UfeHex>")]
            paymaster_data: &'a Vec<FieldElement>,
            #[serde_as(as = "Vec<UfeHex>")]
            account_deployment_data: &'a Vec<FieldElement>,
        }

        let versioned = Versioned {
            version: if self.is_query {
                QUERY_VERSION_THREE
            } else {
                FieldElement::THREE
            },
            contract_class: &self.contract_class,
            compiled_class_hash: &self.compiled_class_hash,
            sender_address: &self.sender_address,
            signature: &self.signature,
            nonce: &self.nonce,
            nonce_data_availability_mode: &self.nonce_data_availability_mode,
            fee_data_availability_mode: &self.fee_data_availability_mode,
            resource_bounds: &self.resource_bounds,
            tip: &self.tip,
            paymaster_data: &self.paymaster_data,
            account_deployment_data: &self.account_deployment_data,
        };

        Versioned::serialize(&versioned, serializer)
    }
}
