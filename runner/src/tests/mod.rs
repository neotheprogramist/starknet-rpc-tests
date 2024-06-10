// src/tests.rs
#[cfg(test)]
mod tests {
    use crate::utils::{
        // accounts::{Call, RawDeclarationV3, RawExecutionV3},
        accounts::RawDeclarationV3,
        codegen::{
            BlockTag, BroadcastedDeclareTransactionV3, DataAvailabilityMode, ResourceBounds,
            ResourceBoundsMapping,
        },
        provider::Provider,
        utils::{create_jsonrpc_client, get_compiled_contract},
        BlockId,
        BroadcastedDeclareTransaction,
    };
    use starknet_crypto::FieldElement;
    use starknet_signers::SigningKey;
    use std::sync::Arc;

    #[tokio::test]
    async fn jsonrpc_get_nonce() {
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
    async fn jsonrpc_add_declare_transaction() {
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
                max_amount: 200000,
                max_price_per_unit: 500000000000000,
            },
            l2_gas: ResourceBounds {
                max_amount: 0,
                max_price_per_unit: 0,
            },
        };

        let signer: SigningKey = SigningKey::from_secret_scalar(
            FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
        );
        let raw_declaration = RawDeclarationV3 {
            contract_class: Arc::new(flattened_sierra_class.clone()),
            compiled_class_hash,
            nonce,
            gas: 200000,
            gas_price: 500000000000000,
        };
        let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();

        let tx_hash = raw_declaration.transaction_hash(chain_id, sender_address, false);
        println!("TX HASH  {}", tx_hash);
        let signature = signer.sign(&tx_hash).unwrap();
        println!("Signature {}", signature);

        let txn: BroadcastedDeclareTransactionV3 = BroadcastedDeclareTransactionV3 {
            sender_address,
            compiled_class_hash,
            signature: vec![signature.r, signature.s],
            nonce,
            contract_class: Arc::new(flattened_sierra_class.clone()),
            resource_bounds,
            is_query: false,
            paymaster_data: vec![],
            account_deployment_data: vec![],
            tip: 0,
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
        };
        let declare_transaction = BroadcastedDeclareTransaction::V3(txn);
        let result = rpc_client
            .add_declare_transaction(declare_transaction)
            .await
            .unwrap();

        println!("RESULT TRANSACTION HASH  {}", result.transaction_hash);
    }

    #[tokio::test]
    async fn jsonrpc_invoke() {
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
                max_amount: 200000,
                max_price_per_unit: 500000000000000,
            },
            l2_gas: ResourceBounds {
                max_amount: 0,
                max_price_per_unit: 0,
            },
        };

        let signer: SigningKey = SigningKey::from_secret_scalar(
            FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
        );

        // let call  = Call {
        //     to:
        //     selector:
        //     calldata:
        // };
        let raw_declaration = RawExecutionV3 {
            calls: vec![call],
            nonce,
            gas: 200000,
            gas_price: 500000000000000,
        };
        let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();

        let tx_hash = raw_declaration.transaction_hash(chain_id, sender_address, false);
        println!("TX HASH  {}", tx_hash);
        let signature = signer.sign(&tx_hash).unwrap();
        println!("Signature {}", signature);

        let txn: BroadcastedDeclareTransactionV3 = BroadcastedDeclareTransactionV3 {
            sender_address,
            compiled_class_hash,
            signature: vec![signature.r, signature.s],
            nonce,
            contract_class: Arc::new(flattened_sierra_class.clone()),
            resource_bounds,
            is_query: false,
            paymaster_data: vec![],
            account_deployment_data: vec![],
            tip: 0,
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
        };
        let declare_transaction = BroadcastedDeclareTransaction::V3(txn);
        let result = rpc_client
            .add_declare_transaction(declare_transaction)
            .await
            .unwrap();

        println!("RESULT TRANSACTION HASH  {}", result.transaction_hash);
    }
}
