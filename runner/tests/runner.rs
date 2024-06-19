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
    codegen::{BlockTag, FunctionCall, MsgFromL1, TransactionExecutionStatus},
    contract::factory::ContractFactory,
    errors::{parse_class_hash_from_error, RunnerError},
    eth_address::EthAddress,
    execution_result::ExecutionResult,
    models::{
        BlockId, ContractClass, DeclareTransaction, InvokeTransaction, InvokeTransactionResult,
        MaybePendingBlockWithReceipts, MaybePendingBlockWithTxs, MaybePendingStateUpdate,
        Transaction, TransactionReceipt, TransactionStatus,
    },
    provider::{Provider, ProviderError},
    starknet_utils::{
        create_jsonrpc_client, get_compiled_contract, get_selector_from_name,
        get_storage_var_address,
    },
    transports::{http::HttpTransport, JsonRpcClient, MaybePendingBlockWithTxHashes},
};

#[tokio::test]
async fn jsonrpc_spec_version() {
    let rpc_client = create_jsonrpc_client();

    let version = rpc_client.spec_version().await.unwrap();

    assert_eq!(version, "0.7.1");
}

#[tokio::test]
async fn jsonrpc_get_nonce() {
    let sender_address = FieldElement::from_hex_be(
        "0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1",
    )
    .unwrap();

    let signer: LocalWallet = LocalWallet::from(SigningKey::from_secret_scalar(
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
async fn jsonrpc_get_block_with_tx_hashes() {
    let rpc_client = create_jsonrpc_client();

    let block = rpc_client
        .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap();

    let block = match block {
        MaybePendingBlockWithTxHashes::Block(block) => block,
        _ => panic!("unexpected block response type"),
    };
    assert_eq!(block.block_number, 0);
}

#[tokio::test]
async fn jsonrpc_get_block_with_txs() {
    let rpc_client = create_jsonrpc_client();

    let block = rpc_client
        .get_block_with_txs(BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap();

    let block = match block {
        MaybePendingBlockWithTxs::Block(block) => block,
        _ => panic!("unexpected block response type"),
    };

    assert_eq!(block.block_number, 0);
}

#[tokio::test]
async fn jsonrpc_get_block_with_receipts() {
    let rpc_client = create_jsonrpc_client();

    let block = rpc_client
        .get_block_with_receipts(BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap();

    let block = match block {
        MaybePendingBlockWithReceipts::Block(block) => block,
        _ => panic!("unexpected block response type"),
    };

    assert_eq!(block.block_number, 0);
}

#[tokio::test]
async fn jsonrpc_get_state_update() {
    let rpc_client = create_jsonrpc_client();

    let state_update = rpc_client
        .get_state_update(BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap();

    let state_update = match state_update {
        MaybePendingStateUpdate::Update(value) => value,
        _ => panic!("unexpected data type"),
    };

    assert_eq!(state_update.new_root, FieldElement::ZERO);
}

#[tokio::test]
async fn jsonrpc_get_storage_at() {
    let rpc_client = create_jsonrpc_client();

    // Checks L2 ETH balance via storage taking advantage of implementation detail
    let eth_balance = rpc_client
        .get_storage_at(
            FieldElement::from_hex_be(
                "049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
            )
            .unwrap(),
            get_storage_var_address(
                "ERC20_balances",
                &[FieldElement::from_hex_be(
                    "03f47d3911396b6d579fd7848cf576286ab6f96dda977915d6c7b10f3dd2315b",
                )
                .unwrap()],
            )
            .unwrap(),
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap();

    assert_eq!(eth_balance, FieldElement::ZERO);
}

#[tokio::test]
async fn jsonrpc_get_transaction_status_succeeded() {
    let rpc_client: JsonRpcClient<HttpTransport> = create_jsonrpc_client();
    let sender_address = FieldElement::from_hex_be(
        "0x4d8bb41636b42d3c69039f3537333581cc19356a0c93904fa3e569498c23ad0",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xb467066159b295a7667b633d6bdaabac").unwrap(),
    ));
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
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

    // Step 2: Deploy the contract
    let deploy_result = deploy_contract_v3(&account, class_hash).await;
    let status = rpc_client
        .get_transaction_status(deploy_result.transaction_hash)
        .await
        .unwrap();

    match status {
        TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded) => {}
        _ => panic!("unexpected transaction status"),
    }
}

#[tokio::test]
async fn jsonrpc_get_transaction_by_hash_invoke_v1() {
    let rpc_client = create_jsonrpc_client();
    let (account, contract_address) = decalare_and_deploy(
        "0x4b3f4ba8c00a02b66142a4b1dd41a4dfab4f92650922a3280977b0f03c75ee1",
        "0x57b2f8431c772e647712ae93cc616638",
        "0x534e5f5345504f4c4941",
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await;
    let amount = FieldElement::from_hex_be("0x10").unwrap();
    let invoke_v1_result = account
        .execute_v1(vec![Call {
            to: contract_address,
            selector: get_selector_from_name("increase_balance").unwrap(),
            calldata: vec![amount],
        }])
        .send()
        .await
        .unwrap();

    let tx = rpc_client
        .get_transaction_by_hash(invoke_v1_result.transaction_hash)
        .await
        .unwrap();

    let tx = match tx {
        Transaction::Invoke(InvokeTransaction::V1(tx)) => tx,
        _ => panic!("unexpected tx response type"),
    };

    assert!(tx.sender_address > FieldElement::ZERO);
}

#[tokio::test]
async fn jsonrpc_get_transaction_by_hash_l1_handler() {
    let rpc_client = create_jsonrpc_client();

    let tx = rpc_client
        .get_transaction_by_hash(
            FieldElement::from_hex_be(
                "0785c2ada3f53fbc66078d47715c27718f92e6e48b96372b36e5197de69b82b5",
            )
            .unwrap(),
        )
        .await
        .unwrap();

    let tx = match tx {
        Transaction::L1Handler(tx) => tx,
        _ => panic!("unexpected tx response type"),
    };

    assert!(tx.entry_point_selector > FieldElement::ZERO);
}

//Network rerun needed for test to succeed, declaration can only occur once
#[tokio::test]
async fn jsonrpc_get_transaction_by_hash_declare_v3() {
    let rpc_client = create_jsonrpc_client();
    let sender_address = FieldElement::from_hex_be(
        "0x3736286f1050d4ba816b4d56d15d80ca74c1752c4e847243f1da726c36e06f",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xa56597ba3378fa9e6440ea9ae0cf2865").unwrap(),
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
    let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
        "../target/dev/sample_SampleStarknet.contract_class.json",
        "../target/dev/sample_SampleStarknet.compiled_contract_class.json",
    )
    .await
    .unwrap();

    let declare_result = account
        .declare_v3(Arc::new(flattened_sierra_class), compiled_class_hash)
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
        .unwrap();

    let tx = rpc_client
        .get_transaction_by_hash(declare_result.transaction_hash)
        .await
        .unwrap();

    let tx = match tx {
        Transaction::Declare(DeclareTransaction::V3(tx)) => tx,
        _ => panic!("unexpected tx response type"),
    };
    dbg!(tx.clone());

    assert!(tx.sender_address > FieldElement::ZERO);
}

#[tokio::test]
async fn jsonrpc_get_class_cairo_1() {
    let sender_address = FieldElement::from_hex_be(
        "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
    ));
    let rpc_client = create_jsonrpc_client();
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let mut account = SingleOwnerAccount::new(
        rpc_client.clone(),
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

    let class = account
        .provider()
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
        .unwrap();

    let class = match class {
        ContractClass::Sierra(class) => class,
        _ => panic!("unexpected class type"),
    };

    assert!(!class.sierra_program.is_empty());
}

#[tokio::test]
async fn jsonrpc_get_class_hash_at() {
    let client = create_jsonrpc_client();
    let sender_address = FieldElement::from_hex_be(
        "0x4d8bb41636b42d3c69039f3537333581cc19356a0c93904fa3e569498c23ad0",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xb467066159b295a7667b633d6bdaabac").unwrap(),
    ));
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let mut account = SingleOwnerAccount::new(
        client.clone(),
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

    let deploy_result = deploy_contract_v3(&account, class_hash).await;

    let receipt = client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => panic!("unexpected receipt response type"),
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => panic!("unexpected execution result"),
    }

    let class_hash_check = account
        .provider()
        .get_class_hash_at(BlockId::Tag(BlockTag::Latest), receipt.contract_address)
        .await
        .unwrap();

    assert_eq!(class_hash_check, class_hash);
}

//XXX
#[tokio::test]
async fn jsonrpc_get_class_at() {
    let client = create_jsonrpc_client();
    let sender_address = FieldElement::from_hex_be(
        "0x4d8bb41636b42d3c69039f3537333581cc19356a0c93904fa3e569498c23ad0",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xb467066159b295a7667b633d6bdaabac").unwrap(),
    ));
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let mut account = SingleOwnerAccount::new(
        client.clone(),
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

    let deploy_result = deploy_contract_v3(&account, class_hash).await;

    let receipt = client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => panic!("unexpected receipt response type"),
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => panic!("unexpected execution result"),
    }

    let class = account
        .provider()
        .get_class_at(BlockId::Tag(BlockTag::Latest), receipt.contract_address)
        .await
        .unwrap();

    let class = match class {
        ContractClass::Sierra(class) => class,
        _ => panic!("unexpected class type"),
    };

    assert!(!class.sierra_program.is_empty());
}
// XXX
#[tokio::test]
async fn jsonrpc_get_block_transaction_count() {
    let rpc_client = create_jsonrpc_client();

    let count = rpc_client
        .get_block_transaction_count(BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap();

    assert_eq!(count, 1);
}

// XXX
#[tokio::test]
async fn jsonrpc_estimate_message_fee() {
    let rpc_client = create_jsonrpc_client();

    let estimate = rpc_client
        .estimate_message_fee(
            MsgFromL1 {
                from_address: EthAddress::from_hex("0x8453FC6Cd1bCfE8D4dFC069C400B433054d47bDc")
                    .unwrap(),
                to_address: FieldElement::from_hex_be(
                    "04c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f",
                )
                .unwrap(),
                entry_point_selector: FieldElement::from_hex_be(
                    "02d757788a8d8d6f21d1cd40bce38a8222d70654214e96ff95d8086e684fbee5",
                )
                .unwrap(),
                payload: vec![FieldElement::ONE, FieldElement::ONE, FieldElement::ONE],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap();

    assert!(estimate.gas_consumed > FieldElement::ZERO);
    assert!(estimate.gas_price > FieldElement::ZERO);
    assert!(estimate.overall_fee > FieldElement::ZERO);
}

///XX
#[tokio::test]
async fn jsonrpc_block_number() {
    let rpc_client = create_jsonrpc_client();

    let block_number = rpc_client.block_number().await.unwrap();
    assert!(block_number > 0);
}

/// XXX
#[tokio::test]
async fn jsonrpc_chain_id() {
    let rpc_client = create_jsonrpc_client();

    let chain_id = rpc_client.chain_id().await.unwrap();
    assert!(chain_id > FieldElement::ZERO);
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
async fn test_increase_and_get_balance() {
    let sender_address = FieldElement::from_hex_be(
        "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0x71d7bb07b9a64f6f78ac4c816aff4da9").unwrap(),
    ));
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let client = create_jsonrpc_client();
    let mut account = SingleOwnerAccount::new(
        client.clone(),
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

    let deploy_result = deploy_contract_v3(&account, class_hash).await;
    let contract_address = deploy_result.transaction_hash;

    let increase_amount = FieldElement::from_dec_str("10").unwrap();
    account
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

    let call_result: Vec<FieldElement> = client
        .call(
            FunctionCall {
                contract_address: contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![FieldElement::from_hex_be(
                    "YOUR_ACCOUNT_CONTRACT_ADDRESS_IN_HEX_HERE",
                )
                .unwrap()],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .expect("failed to call contract");

    let expected_balance = FieldElement::from_dec_str("1").unwrap(); // Assuming `Positive` is encoded as `1`
    assert_eq!(call_result[0], expected_balance);
}

#[tokio::test]
async fn jsonrpc_call() {
    let client = create_jsonrpc_client();
    let sender_address = FieldElement::from_hex_be(
        "0x4d8bb41636b42d3c69039f3537333581cc19356a0c93904fa3e569498c23ad0",
    )
    .unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0xb467066159b295a7667b633d6bdaabac").unwrap(),
    ));
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let mut account = SingleOwnerAccount::new(
        client.clone(),
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

    let deploy_result = deploy_contract_v3(&account, class_hash).await;

    let receipt = client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => panic!("unexpected receipt response type"),
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => panic!("unexpected execution result"),
    }

    let eth_balance = client
        .call(
            &FunctionCall {
                contract_address: receipt.contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap();

    println!("BALANCE :{}", eth_balance[0]);
    assert!(eth_balance[0] > FieldElement::ZERO);
}

#[tokio::test]
async fn jsonrpc_get_transaction_receipt_deploy() {
    let client = create_jsonrpc_client();
    let sender_address = FieldElement::from_hex_be(
        "0x557ba9ef60b52dad611d79b60563901458f2476a5c1002a8b4869fcb6654c7e",
    )
    .unwrap();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be("0x15b5e3013d752c909988204714f1ff35").unwrap(),
    ));
    let chain_id = FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap();
    let mut account = SingleOwnerAccount::new(
        client.clone(),
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

    let deploy_result = deploy_contract_v3(&account, class_hash).await;

    let receipt = client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await
        .unwrap();

    assert!(receipt.block.is_block());

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => panic!("unexpected receipt response type"),
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => panic!("unexpected execution result"),
    }
}

#[tokio::test]
async fn jsonrpc_invoke() {
    let (account, contract_address) = decalare_and_deploy(
        "0x4b3f4ba8c00a02b66142a4b1dd41a4dfab4f92650922a3280977b0f03c75ee1",
        "0x57b2f8431c772e647712ae93cc616638",
        "0x534e5f5345504f4c4941",
        "../target/dev/example_HelloStarknet.contract_class.json",
        "../target/dev/example_HelloStarknet.compiled_contract_class.json",
    )
    .await;

    let amount = FieldElement::from_hex_be("0x10").unwrap();
    account
        .execute_v3(vec![Call {
            to: contract_address,
            selector: get_selector_from_name("increase_balance").unwrap(),
            calldata: vec![amount],
        }])
        .gas(200000)
        .gas_price(500000000000000)
        .send()
        .await
        .unwrap();
    let eth_balance = account
        .provider()
        .call(
            &FunctionCall {
                contract_address: contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap();

    assert_eq!(eth_balance[0], FieldElement::ZERO);
}

//helper function, reused a lot
pub async fn decalare_and_deploy(
    sender_address: &str,
    private_key: &str,
    chain_id: &str,
    sierra_path: &str,
    casm_path: &str,
) -> (
    SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    FieldElement,
) {
    let client = create_jsonrpc_client();
    let address = FieldElement::from_hex_be(sender_address).unwrap();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(private_key).unwrap(),
    ));
    let chain_id = FieldElement::from_hex_be(chain_id).unwrap();
    let mut account = SingleOwnerAccount::new(
        client.clone(),
        signer,
        address,
        chain_id,
        ExecutionEncoding::New,
    );
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let class_hash = declare_contract_v3(&account, sierra_path, casm_path)
        .await
        .unwrap();
    let deploy_result = deploy_contract_v3(&account, class_hash).await;
    let receipt = client
        .get_transaction_receipt(deploy_result.transaction_hash)
        .await
        .unwrap();
    assert!(receipt.block.is_block());

    let receipt = match receipt.receipt {
        TransactionReceipt::Deploy(receipt) => receipt,
        _ => panic!("unexpected receipt response type"),
    };

    match receipt.execution_result {
        ExecutionResult::Succeeded => {}
        _ => panic!("unexpected execution result"),
    }
    (account, receipt.contract_address)
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
