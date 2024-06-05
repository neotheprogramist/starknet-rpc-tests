use crate::utils::serde_impls::NumAsHex;
use crate::utils::unsigned_field_element::UfeHex;
use crate::BroadcastedDeclareTransaction;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use starknet::core::types::{FieldElement, FlattenedSierraClass};
use std::sync;
/// Sync status.
///
///
/// An object describing the node synchronization status.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct SyncStatus {
    /// The hash of the block from which the sync started
    #[serde_as(as = "UfeHex")]
    pub starting_block_hash: FieldElement,
    /// The number (height) of the block from which the sync started
    pub starting_block_num: u64,
    /// The hash of the current block being synchronized
    #[serde_as(as = "UfeHex")]
    pub current_block_hash: FieldElement,
    /// The number (height) of the current block being synchronized
    pub current_block_num: u64,
    /// The hash of the estimated highest block to be synchronized
    #[serde_as(as = "UfeHex")]
    pub highest_block_hash: FieldElement,
    /// The number (height) of the estimated highest block to be synchronized
    pub highest_block_num: u64,
}
#[cfg(target_has_atomic = "ptr")]
pub type OwnedPtr<T> = sync::Arc<T>;
#[cfg(not(target_has_atomic = "ptr"))]
pub type OwnedPtr<T> = alloc::boxed::Box<T>;

// Broadcasted declare transaction v3.
///
/// Broadcasted declare contract transaction v3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastedDeclareTransactionV3 {
    /// The address of the account contract sending the declaration transaction
    pub sender_address: FieldElement,
    /// The hash of the cairo assembly resulting from the sierra compilation
    pub compiled_class_hash: FieldElement,
    /// Signature
    pub signature: Vec<FieldElement>,
    /// Nonce
    pub nonce: FieldElement,
    /// The class to be declared
    pub contract_class: OwnedPtr<FlattenedSierraClass>,
    /// Resource bounds for the transaction execution
    pub resource_bounds: ResourceBoundsMapping,
    /// The tip for the transaction
    pub tip: u64,
    /// Data needed to allow the paymaster to pay for the transaction in native tokens
    pub paymaster_data: Vec<FieldElement>,
    /// Data needed to deploy the account contract from which this tx will be initiated
    pub account_deployment_data: Vec<FieldElement>,
    /// The storage domain of the account's nonce (an account has a nonce per da mode)
    pub nonce_data_availability_mode: DataAvailabilityMode,
    /// The storage domain of the account's balance from which fee will be charged
    pub fee_data_availability_mode: DataAvailabilityMode,
    /// If set to `true`, uses a query-only transaction version that's invalid for execution
    pub is_query: bool,
}

impl Serialize for BroadcastedDeclareTransactionV3 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[serde_as]
        #[derive(Serialize)]
        struct Tagged<'a> {
            pub r#type: &'a str,
            #[serde_as(as = "UfeHex")]
            pub sender_address: &'a FieldElement,
            #[serde_as(as = "UfeHex")]
            pub compiled_class_hash: &'a FieldElement,
            #[serde_as(as = "UfeHex")]
            pub version: &'a FieldElement,
            #[serde_as(as = "[UfeHex]")]
            pub signature: &'a [FieldElement],
            #[serde_as(as = "UfeHex")]
            pub nonce: &'a FieldElement,
            pub contract_class: &'a FlattenedSierraClass,
            pub resource_bounds: &'a ResourceBoundsMapping,
            #[serde_as(as = "NumAsHex")]
            pub tip: &'a u64,
            #[serde_as(as = "[UfeHex]")]
            pub paymaster_data: &'a [FieldElement],
            #[serde_as(as = "[UfeHex]")]
            pub account_deployment_data: &'a [FieldElement],
            pub nonce_data_availability_mode: &'a DataAvailabilityMode,
            pub fee_data_availability_mode: &'a DataAvailabilityMode,
        }

        let r#type = "DECLARE";

        let version = &(if self.is_query {
            FieldElement::THREE + QUERY_VERSION_OFFSET
        } else {
            FieldElement::THREE
        });

        let tagged = Tagged {
            r#type,
            sender_address: &self.sender_address,
            compiled_class_hash: &self.compiled_class_hash,
            version,
            signature: &self.signature,
            nonce: &self.nonce,
            contract_class: &self.contract_class,
            resource_bounds: &self.resource_bounds,
            tip: &self.tip,
            paymaster_data: &self.paymaster_data,
            account_deployment_data: &self.account_deployment_data,
            nonce_data_availability_mode: &self.nonce_data_availability_mode,
            fee_data_availability_mode: &self.fee_data_availability_mode,
        };

        Tagged::serialize(&tagged, serializer)
    }
}

impl<'de> Deserialize<'de> for BroadcastedDeclareTransactionV3 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[serde_as]
        #[derive(Deserialize)]
        #[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
        struct Tagged {
            pub r#type: Option<String>,
            #[serde_as(as = "UfeHex")]
            pub sender_address: FieldElement,
            #[serde_as(as = "UfeHex")]
            pub compiled_class_hash: FieldElement,
            #[serde_as(as = "UfeHex")]
            pub version: FieldElement,
            #[serde_as(as = "Vec<UfeHex>")]
            pub signature: Vec<FieldElement>,
            #[serde_as(as = "UfeHex")]
            pub nonce: FieldElement,
            pub contract_class: FlattenedSierraClass,
            pub resource_bounds: ResourceBoundsMapping,
            #[serde_as(as = "NumAsHex")]
            pub tip: u64,
            #[serde_as(as = "Vec<UfeHex>")]
            pub paymaster_data: Vec<FieldElement>,
            #[serde_as(as = "Vec<UfeHex>")]
            pub account_deployment_data: Vec<FieldElement>,
            pub nonce_data_availability_mode: DataAvailabilityMode,
            pub fee_data_availability_mode: DataAvailabilityMode,
        }

        let tagged = Tagged::deserialize(deserializer)?;

        if let Some(tag_field) = &tagged.r#type {
            if tag_field != "DECLARE" {
                return Err(serde::de::Error::custom("invalid `type` value"));
            }
        }

        let is_query = if tagged.version == FieldElement::THREE {
            false
        } else if tagged.version == FieldElement::THREE + QUERY_VERSION_OFFSET {
            true
        } else {
            return Err(serde::de::Error::custom("invalid `version` value"));
        };

        Ok(Self {
            sender_address: tagged.sender_address,
            compiled_class_hash: tagged.compiled_class_hash,
            signature: tagged.signature,
            nonce: tagged.nonce,
            contract_class: OwnedPtr::new(tagged.contract_class),
            resource_bounds: tagged.resource_bounds,
            tip: tagged.tip,
            paymaster_data: tagged.paymaster_data,
            account_deployment_data: tagged.account_deployment_data,
            nonce_data_availability_mode: tagged.nonce_data_availability_mode,
            fee_data_availability_mode: tagged.fee_data_availability_mode,
            is_query,
        })
    }
}

const QUERY_VERSION_OFFSET: FieldElement = FieldElement::from_mont([
    18446744073700081665,
    17407,
    18446744073709551584,
    576460752142434320,
]);

#[derive(Serialize, Deserialize)]
pub struct DeclareTransactionV3 {
    pub transaction_hash: FieldElement,
    pub sender_address: FieldElement,
    pub compiled_class_hash: FieldElement,
    pub signature: Vec<FieldElement>,
    pub nonce: FieldElement,
    pub class_hash: FieldElement,
    pub resource_bounds: ResourceBoundsMapping,
    pub tip: u64,
    pub paymaster_data: Vec<FieldElement>,
    pub account_deployment_data: Vec<FieldElement>,
    pub nonce_data_availability_mode: DataAvailabilityMode,
    pub fee_data_availability_mode: DataAvailabilityMode,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataAvailabilityMode {
    #[serde(rename = "L1")]
    L1,
    #[serde(rename = "L2")]
    L2,
}
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct SierraEntryPoint {
    /// A unique identifier of the entry point (function) in the program
    #[serde_as(as = "UfeHex")]
    pub selector: FieldElement,
    /// The index of the function in the program
    pub function_idx: u64,
}
/// Entry points by type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct EntryPointsByType {
    /// Constructor
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: Vec<SierraEntryPoint>,
    /// External
    #[serde(rename = "EXTERNAL")]
    pub external: Vec<SierraEntryPoint>,
    /// L1 handler
    #[serde(rename = "L1_HANDLER")]
    pub l1_handler: Vec<SierraEntryPoint>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct ResourceBoundsMapping {
    /// The max amount and max price per unit of L1 gas used in this tx
    pub l1_gas: ResourceBounds,
    /// The max amount and max price per unit of L2 gas used in this tx
    pub l2_gas: ResourceBounds,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct ResourceBounds {
    /// The max amount of the resource that can be used in the tx
    #[serde_as(as = "NumAsHex")]
    pub max_amount: u64,
    /// The max price per unit of this resource for this tx
    #[serde_as(as = "NumAsHex")]
    pub max_price_per_unit: u128,
}

/// Reference version of [AddDeclareTransactionRequest].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AddDeclareTransactionRequestRef<'a> {
    pub declare_transaction: &'a BroadcastedDeclareTransaction,
}
