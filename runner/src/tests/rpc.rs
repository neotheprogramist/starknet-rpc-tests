use starknet::signers::SigningKey;

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

    assert_eq!(nonce, FieldElement::ONE);
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

    let private_key: SigningKey = SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
    );
    let signature = private_key.sign(&compiled_class_hash).unwrap();

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
