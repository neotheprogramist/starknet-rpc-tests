use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use starknet_crypto::FieldElement;
use starknet_signers::{LocalWallet, SigningKey};
use std::sync::Arc;
use utils::{
    account::{
        call::Call,
        single_owner::{ExecutionEncoding, SingleOwnerAccount},
        Account, AccountError, ConnectedAccount,
    },
    codegen::BlockTag,
    contract::factory::ContractFactory,
    errors::{parse_class_hash_from_error, RunnerError},
    models::{BlockId, InvokeTransactionResult},
    provider::{Provider, ProviderError},
    starknet_utils::{create_jsonrpc_client, get_compiled_contract, get_selector_from_name},
};

#[tokio::test]
async fn jsonrpc_get_nonce() {
    let sender_address = FieldElement::from_hex_be(
        "0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xe1406455b7d66b1690803be066cbe5e").unwrap(),
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
    let nonce: FieldElement = account.get_nonce().await.unwrap();
    assert_eq!(nonce, FieldElement::ZERO)
}

#[tokio::test]
async fn jsonrpc_add_declare_transaction() {
    let sender_address = FieldElement::from_hex_be(
        "0x49dfb8ce986e21d354ac93ea65e6a11f639c1934ea253e5ff14ca62eca0f38e",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xa20a02f0ac53692d144b20cb371a60d7").unwrap(),
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

    declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn jsonrpc_deploy() {
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

    let class_hash = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await
    .unwrap();

    let random_loop_count = rand::thread_rng().gen_range(10..=30);

    for _ in 0..random_loop_count {
        deploy_contract_v3(&account, class_hash).await;
    }

    let nonce = account.get_nonce().await.unwrap();
    assert_eq!(
        nonce,
        FieldElement::from_dec_str(&(random_loop_count + 1).to_string()).unwrap()
    )
}

#[tokio::test]
async fn jsonrpc_invoke() {
    let sender_address = FieldElement::from_hex_be(
        "0x1e8c6c17efa3a047506c0b1610bd188aa3e3dd6c5d9227549b65428de24de78",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0x836203aceb0e9b0066138c321dda5ae6").unwrap(),
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

    let class_hash = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await
    .unwrap();
    let result = invoke_v3(&account, class_hash, "transfer").await;
    tracing::debug!("TRANSACTION HASH {}", result.transaction_hash);
}

#[tokio::test]
async fn test_increase_and_get_balance() {
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

    // Step 1: Declare the contract
    let class_hash = declare_contract_v3(
        &account,
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await
    .unwrap();

    // Step 2: Deploy the contract
    let deploy_result = deploy_contract_v3(&account, class_hash).await;
    let contract_address = deploy_result.transaction_hash;

    // Step 3: Invoke the `increase_balance` function
    let increase_amount = FieldElement::from_dec_str("10").unwrap();
    let result = account
        .execute_v3(vec![Call {
            to: contract_address,
            selector: get_selector_from_name("increase_balance").unwrap(),
            calldata: vec![increase_amount],
        }])
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
        .unwrap();

    // Step 4: Call the `get_balance` function and assert the balance
    let call_result: Vec<FieldElement> = account
        .execute_v3(Call {
            to: contract_address,
            selector: get_selector_from_name("get_balance").unwrap(),
            calldata: vec![],
        })
        .await
        .unwrap();

    // The expected balance result is `Positive` which should be encoded as a specific FieldElement
    let expected_balance = FieldElement::from_dec_str("1").unwrap(); // Assuming `Positive` is encoded as `1`
    assert_eq!(call_result[0], expected_balance);
}

pub async fn declare_contract_v3<P: Provider + Send + Sync>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    sierra_path: &str,
    casm_path: &str,
) -> Result<FieldElement, RunnerError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    match account
        .declare_v3(Arc::new(flattened_sierra_class), compiled_class_hash)
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                )))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
            } else {
                Err(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    starkneterror
                )))
            }
        }
        Err(e) => {
            tracing::info!("General account error encountered: {:?}, possible cause - incorrect address or public_key in environment variables!", e);
            Err(RunnerError::AccountFailure(format!("Account error: {}", e)))
        }
    }
}

pub async fn invoke_v3<P: Provider + Send + Sync>(
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
pub async fn deploy_contract_v3<P: Provider + Send + Sync>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    class_hash: FieldElement,
) -> InvokeTransactionResult {
    let factory = ContractFactory::new(class_hash, account);
    let mut salt_buffer = [0u8; 32];
    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut salt_buffer[1..]);

    let result = factory
        .deploy_v3(
            vec![],
            FieldElement::from_bytes_be(&salt_buffer).unwrap(),
            true,
        )
        .send()
        .await
        .unwrap();
    result
}
