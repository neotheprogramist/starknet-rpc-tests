// src/tests.rs
#[cfg(test)]
mod tests {
    use crate::transports::http::HttpTransport;
    use crate::transports::JsonRpcClient;
    use crate::utils::account::Account;
    use crate::utils::starknet_utils::get_selector_from_name;
    use crate::utils::{
        // accounts::{Call, RawDeclarationV3, RawExecutionV3},
        account::{
            call::Call,
            single_owner::{ExecutionEncoding, SingleOwnerAccount},
        },
        codegen::{
            BlockTag, BroadcastedDeclareTransactionV3, DataAvailabilityMode, ResourceBounds,
            ResourceBoundsMapping,
        },
        provider::Provider,
        starknet_utils::{create_jsonrpc_client, get_compiled_contract},
        BlockId,
        BroadcastedDeclareTransaction,
    };
    use crate::utils::{DeclareTransactionResult, InvokeTransactionResult};
    use starknet_crypto::FieldElement;
    use starknet_signers::{LocalWallet, SigningKey};
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

    async fn declare_contract_v3<P: Provider + Send + Sync>(
        account: &SingleOwnerAccount<P, LocalWallet>,
        sierra_path: &str,
        casm_path: &str,
    ) -> DeclareTransactionResult {
        let (flattened_sierra_class, compiled_class_hash) =
            get_compiled_contract(sierra_path, casm_path).await.unwrap();

        let result = account
            .declare_v3(Arc::new(flattened_sierra_class), compiled_class_hash)
            .gas(200000)
            .gas_price(500000000000000)
            .send()
            .await
            .unwrap();
        result
    }

    async fn invoke_v3<P: Provider + Send + Sync>(
        account: &SingleOwnerAccount<P, LocalWallet>,
        to: FieldElement,
        method: &str,
    ) -> InvokeTransactionResult {
        let result = account
            .execute_v3(vec![Call {
                to,
                selector: get_selector_from_name(method).unwrap(),
                calldata: vec![
                    FieldElement::from_hex_be("0x1234").unwrap(),
                    FieldElement::ONE,
                    FieldElement::ZERO,
                ],
            }])
            .gas(200000)
            .gas_price(500000000000000)
            .send()
            .await
            .unwrap();

        result
    }

    #[tokio::test]
    async fn jsonrpc_add_declare_transaction() {
        let sender_address = FieldElement::from_hex_be(
            "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
        )
        .unwrap();

        let signer = LocalWallet::from(SigningKey::from_secret_scalar(
            FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
        ));
        let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
        let mut account = SingleOwnerAccount::new(
            create_jsonrpc_client(),
            signer,
            sender_address,
            chain_id,
            ExecutionEncoding::New,
        );
        account.set_block_id(BlockId::Tag(BlockTag::Pending));

        let declare_transaction_result = declare_contract_v3(
            &account,
            "../target/dev/example_HelloStarknet.contract_class.json",
            "../target/dev/example_HelloStarknet.compiled_contract_class.json",
        )
        .await;
        println!(
            "RESULT TRANSACTION HASH  {}",
            declare_transaction_result.transaction_hash
        );
    }

    #[tokio::test]
    async fn jsonrpc_invoke() {
        let sender_address = FieldElement::from_hex_be(
            "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
        )
        .unwrap();

        let signer = LocalWallet::from(SigningKey::from_secret_scalar(
            FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
        ));
        let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
        let mut account = SingleOwnerAccount::new(
            create_jsonrpc_client(),
            signer,
            sender_address,
            chain_id,
            ExecutionEncoding::New,
        );
        account.set_block_id(BlockId::Tag(BlockTag::Pending));

        let declare_transaction_result = declare_contract_v3(
            &account,
            "../target/dev/example_HelloStarknet.contract_class.json",
            "../target/dev/example_HelloStarknet.compiled_contract_class.json",
        )
        .await;
        let result = invoke_v3(&account, declare_transaction_result.class_hash, "transfer").await;
        tracing::debug!("TRANSACTION HASH {}", result.transaction_hash);
    }
}
