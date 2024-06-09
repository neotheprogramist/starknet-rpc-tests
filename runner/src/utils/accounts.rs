use std::sync::Arc;

use super::codegen::FlattenedSierraClass;
use starknet_crypto::{FieldElement, PoseidonHasher};
/// [DeclarationV3] but with `nonce`, `gas` and `gas_price` already determined.
#[derive(Debug)]
pub struct RawDeclarationV3 {
    pub contract_class: Arc<FlattenedSierraClass>,
    pub compiled_class_hash: FieldElement,
    pub nonce: FieldElement,
    pub gas: u64,
    pub gas_price: u128,
}
/// Cairo string for "declare"
const PREFIX_DECLARE: FieldElement = FieldElement::from_mont([
    17542456862011667323,
    18446744073709551615,
    18446744073709551615,
    191557713328401194,
]);

/// 2 ^ 128 + 1
const QUERY_VERSION_ONE: FieldElement = FieldElement::from_mont([
    18446744073700081633,
    17407,
    18446744073709551584,
    576460752142433776,
]);

/// 2 ^ 128 + 2
const QUERY_VERSION_TWO: FieldElement = FieldElement::from_mont([
    18446744073700081601,
    17407,
    18446744073709551584,
    576460752142433232,
]);

/// 2 ^ 128 + 3
const QUERY_VERSION_THREE: FieldElement = FieldElement::from_mont([
    18446744073700081569,
    17407,
    18446744073709551584,
    576460752142432688,
]);

impl RawDeclarationV3 {
    pub fn transaction_hash(
        &self,
        chain_id: FieldElement,
        address: FieldElement,
        query_only: bool,
    ) -> FieldElement {
        let mut hasher = PoseidonHasher::new();

        hasher.update(PREFIX_DECLARE);
        hasher.update(if query_only {
            QUERY_VERSION_THREE
        } else {
            FieldElement::THREE
        });
        hasher.update(address);

        hasher.update({
            let mut fee_hasher = PoseidonHasher::new();

            // Tip: fee market has not been been activated yet so it's hard-coded to be 0
            fee_hasher.update(FieldElement::ZERO);

            let mut resource_buffer = [
                0, 0, b'L', b'1', b'_', b'G', b'A', b'S', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            resource_buffer[8..(8 + 8)].copy_from_slice(&self.gas.to_be_bytes());
            resource_buffer[(8 + 8)..].copy_from_slice(&self.gas_price.to_be_bytes());
            fee_hasher.update(FieldElement::from_bytes_be(&resource_buffer).unwrap());

            // L2 resources are hard-coded to 0
            let resource_buffer = [
                0, 0, b'L', b'2', b'_', b'G', b'A', b'S', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            fee_hasher.update(FieldElement::from_bytes_be(&resource_buffer).unwrap());

            fee_hasher.finalize()
        });

        // Hard-coded empty `paymaster_data`
        hasher.update(PoseidonHasher::new().finalize());

        hasher.update(chain_id);
        hasher.update(self.nonce);

        // Hard-coded L1 DA mode for nonce and fee
        hasher.update(FieldElement::ZERO);

        // Hard-coded empty `account_deployment_data`
        hasher.update(PoseidonHasher::new().finalize());

        hasher.update(self.contract_class.class_hash());
        hasher.update(self.compiled_class_hash);

        hasher.finalize()
    }

    pub fn contract_class(&self) -> &FlattenedSierraClass {
        &self.contract_class
    }

    pub fn compiled_class_hash(&self) -> FieldElement {
        self.compiled_class_hash
    }

    pub fn nonce(&self) -> FieldElement {
        self.nonce
    }

    pub fn gas(&self) -> u64 {
        self.gas
    }

    pub fn gas_price(&self) -> u128 {
        self.gas_price
    }
}
