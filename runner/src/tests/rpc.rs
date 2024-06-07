use std::sync::Arc;

use starknet::{
    core::types::{FieldElement, FlattenedSierraClass},
    signers::SigningKey,
};

#[tokio::test]
async fn jsonrpc_get_nonce() {
    use crate::utils::provider::Provider;
    use starknet::core::types::FieldElement;

    use crate::{
        create_jsonrpc_client,
        utils::{codegen::BlockTag, BlockId},
    };
    let rpc_client = create_jsonrpc_client();

    let nonce = rpc_client
        .get_nonce(
            BlockId::Tag(BlockTag::Latest),
            FieldElement::from_hex_be(
                "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
            )
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(nonce, FieldElement::ZERO);
}

#[tokio::test]
async fn jsonrpc_add_declare_tranasaction() {
    use crate::utils::provider::Provider;
    use crate::{
        get_compiled_contract,
        utils::{
            codegen::{
                BroadcastedDeclareTransactionV3, DataAvailabilityMode, ResourceBounds,
                ResourceBoundsMapping,
            },
            BroadcastedDeclareTransaction,
        },
    };
    use starknet::core::crypto::pedersen_hash;
    use starknet::core::types::FieldElement;
    use std::sync::Arc;

    use crate::{
        create_jsonrpc_client,
        utils::{codegen::BlockTag, BlockId},
    };
    let rpc_client = create_jsonrpc_client();

    let sender_address = FieldElement::from_hex_be(
        "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
    )
    .unwrap();
    let nonce = rpc_client
        .get_nonce(BlockId::Tag(BlockTag::Latest), sender_address)
        .await
        .unwrap();

    assert_eq!(nonce, FieldElement::ZERO);
    let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await
    .unwrap();
    let resource_bounds = ResourceBoundsMapping {
        l1_gas: ResourceBounds {
            max_amount: 200,
            max_price_per_unit: 100000000000,
        },
        l2_gas: ResourceBounds {
            max_amount: u64::MAX,
            max_price_per_unit: 0,
        },
    };

    let hash2 = pedersen_hash(&compiled_class_hash, &sender_address);

    // Sign the hash using the private key

    let signer: SigningKey = SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
    );
    let raw_declaration = RawDeclarationV3{
        contract_class: Arc::new(flattened_sierra_class),
        compiled_class_hash: compiled_class_hash,
        nonce,
        gas: 0,
        gas_price: 0,
    }

    let tx_hash = raw_declaration.transaction_hash(self.chain_id, self.address, false);
    let signature = signer.sign(&tx_hash).unwrap();

    let txn: BroadcastedDeclareTransactionV3 = BroadcastedDeclareTransactionV3 {
        sender_address,
        compiled_class_hash,
        signature: vec![signature.r, signature.s],
        nonce,
        contract_class: Arc::new(flattened_sierra_class),
        resource_bounds,
        is_query: false,
        paymaster_data: Vec::new(),
        account_deployment_data: Vec::new(),
        tip: 0,
        nonce_data_availability_mode: DataAvailabilityMode::L2,
        fee_data_availability_mode: DataAvailabilityMode::L2,
    };
    let declare_transaction = BroadcastedDeclareTransaction::V3(txn);
    rpc_client
        .add_declare_transaction(declare_transaction)
        .await
        .unwrap();
}

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