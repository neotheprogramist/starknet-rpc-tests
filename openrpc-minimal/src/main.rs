use core::panic;
use std::{path::PathBuf, str::FromStr};

use openrpc_testgen::{
    utils::{
        random_single_owner_account::RandomSingleOwnerAccount,
        v7::{
            accounts::{
                account::{Account, AccountError, ConnectedAccount},
                call::Call,
                creation::helpers::get_chain_id,
                single_owner::{ExecutionEncoding, SingleOwnerAccount},
            },
            contract::factory::ContractFactory,
            endpoints::{
                declare_contract::{
                    extract_class_hash_from_error, get_compiled_contract,
                    parse_class_hash_from_error, RunnerError,
                },
                errors::RpcError,
                utils::{get_selector_from_name, wait_for_sent_transaction},
            },
            providers::{
                jsonrpc::{HttpTransport, JsonRpcClient},
                provider::{Provider, ProviderError},
            },
            signers::{key_pair::SigningKey, local_wallet::LocalWallet},
        },
    },
    RandomizableAccountsTrait,
};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{
    Address, BlockId, BlockTag, EventFilterWithPageRequest, MaybePendingBlockWithTxHashes,
    TxnReceipt,
};
use url::Url;

#[tokio::main]
async fn main() {
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::from_str("http://0.0.0.0:9545/rpc/v0_7").unwrap(),
    ));
    let chain_id = get_chain_id(&provider).await.unwrap();

    let mut random_account = RandomSingleOwnerAccount::new(vec![SingleOwnerAccount::new(
        JsonRpcClient::new(HttpTransport::new(
            Url::from_str("http://0.0.0.0:9545/rpc/v0_7").unwrap(),
        )),
        LocalWallet::from(SigningKey::from_secret_scalar(
            Felt::from_hex("0x07a35c28ab710d0313025c2002f044490a4873832172e45f4f50d90858fa90bf")
                .unwrap(),
        )),
        Felt::from_hex("0x03C159ed30e1e862320D2838afED9d15980b01Dc5184f224B79A63435E5400Be")
            .unwrap(),
        chain_id,
        ExecutionEncoding::New,
    )]);

    random_account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let mut continuation_token = Option::None;
    loop {
        let events_chunk = random_account
            .provider()
            .get_events(EventFilterWithPageRequest {
                address: Option::None,
                from_block: Some(BlockId::Number(322421)),
                to_block: Some(BlockId::Number(322421)),
                keys: Some(vec![vec![]]),
                chunk_size: 100,
                continuation_token,
            })
            .await
            .unwrap();

        // Przetwórz zdarzenia
        for event in events_chunk.events {
            println!("{:?}", event);
        }

        // Jeśli jest continuation_token, przejdź do następnej strony
        if let Some(token) = events_chunk.continuation_token {
            continuation_token = Some(token);
        } else {
            break;
        }
    }

    // let (flattened_sierra_class_1, compiled_class_hash_1) = get_compiled_contract(
    //     PathBuf::from_str("target/dev/contracts_contracts_smpl1_HelloStarknet.contract_class.json")
    //         .unwrap(),
    //     PathBuf::from_str(
    //         "target/dev/contracts_contracts_smpl1_HelloStarknet.compiled_contract_class.json",
    //     )
    //     .unwrap(),
    // )
    // .await
    // .unwrap();

    // // --> DECLARE V3

    // println!("Nonces: {:?}", get_nonces(random_account.clone()).await);

    // let declaration_hash = match random_account
    //     .declare_v3(flattened_sierra_class_1, compiled_class_hash_1)
    //     .send()
    //     .await
    // {
    //     Ok(result) => {
    //         wait_for_sent_transaction(
    //             result.transaction_hash,
    //             &random_account.random_accounts().unwrap(),
    //         )
    //         .await
    //         .unwrap();

    //         Ok(result.class_hash)
    //     }
    //     Err(AccountError::Signing(sign_error)) => {
    //         if sign_error.to_string().contains("is already declared") {
    //             Ok(parse_class_hash_from_error(&sign_error.to_string()).unwrap())
    //         } else {
    //             Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
    //                 "Transaction execution error: {}",
    //                 sign_error
    //             ))))
    //         }
    //     }

    //     Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
    //         if starkneterror.to_string().contains("is already declared") {
    //             Ok(parse_class_hash_from_error(&starkneterror.to_string()).unwrap())
    //         } else {
    //             Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
    //                 "Transaction execution error: {}",
    //                 starkneterror
    //             ))))
    //         }
    //     }
    //     Err(e) => {
    //         let full_error_message = format!("{:?}", e);
    //         println!("full_error_message {:?}", full_error_message);
    //         if full_error_message.contains("is already declared") {
    //             println!("e.to_string {:?}", e.to_string());

    //             println!(
    //                 "No panic, gonna retrieve class hash :) error msg{:?}",
    //                 full_error_message
    //             );
    //             Ok(extract_class_hash_from_error(&full_error_message).unwrap())
    //         } else {
    //             let full_error_message = format!("{:?}", e);
    //             println!("GONNA PANIC!!! error msg{:?}", full_error_message);
    //             panic!("err {:?}", full_error_message);
    //         }
    //     }
    // }
    // .unwrap();

    // println!(
    //     "After Declare V3 --> Nonces: {:?}",
    //     get_nonces(random_account.clone()).await
    // );

    // // --> DEPLOY V3

    // let factory = ContractFactory::new(declaration_hash, random_account.random_accounts().unwrap());
    // let mut salt_buffer = [0u8; 32];
    // let mut rng = StdRng::from_entropy();
    // rng.fill_bytes(&mut salt_buffer[1..]);

    // let deployment_result = factory
    //     .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
    //     .send()
    //     .await
    //     .unwrap();

    // println!(
    //     "\n Deployment tx hash {:?}\n",
    //     deployment_result.transaction_hash
    // );

    // wait_for_sent_transaction(
    //     deployment_result.transaction_hash,
    //     &random_account.random_accounts().unwrap(),
    // )
    // .await
    // .unwrap();

    // println!(
    //     "\n\n\n OUR TXN {:?} \n\n\n",
    //     deployment_result.transaction_hash
    // );

    // // Check if the transaction is in the pending block by transaction hashes
    // match provider
    //     .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Pending))
    //     .await
    //     .unwrap()
    // {
    //     MaybePendingBlockWithTxHashes::Pending(block) => {
    //         println!("\nChecking Pending Block (via hashes):");
    //         let mut found_in_pending = false;

    //         for txn_hash in &block.transactions {
    //             if txn_hash == &deployment_result.transaction_hash {
    //                 println!(
    //                     "✅ Transaction found in Pending Block (via hashes): {:?}",
    //                     txn_hash
    //                 );
    //                 found_in_pending = true;
    //                 break;
    //             } else {
    //                 println!("🔍 Transaction Hash in Pending Block: {:?}", txn_hash);
    //             }
    //         }

    //         if !found_in_pending {
    //             println!("❌ Transaction not found in Pending Block (via hashes).");
    //         }
    //     }
    //     _ => {
    //         panic!("Unexpected block type while checking pending transactions via hashes.");
    //     }
    // }

    // // Check if the transaction is in the latest block by transaction hashes
    // match provider
    //     .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
    //     .await
    //     .unwrap()
    // {
    //     MaybePendingBlockWithTxHashes::Block(block) => {
    //         println!("\nChecking Latest Block (via hashes):");
    //         let mut found_in_latest = false;

    //         for txn_hash in &block.transactions {
    //             if txn_hash == &deployment_result.transaction_hash {
    //                 println!(
    //                     "✅ Transaction found in Latest Block (via hashes): {:?}",
    //                     txn_hash
    //                 );
    //                 found_in_latest = true;
    //                 break;
    //             } else {
    //                 println!("🔍 Transaction Hash in Latest Block: {:?}", txn_hash);
    //             }
    //         }

    //         if !found_in_latest {
    //             println!("❌ Transaction not found in Latest Block (via hashes).");
    //         }
    //     }
    //     _ => {
    //         panic!("Unexpected block type while checking latest transactions via hashes.");
    //     }
    // }

    // println!("\ngetting deployment receipt... can fail!\n");
    // let deployment_receipt = loop {
    //     match random_account
    //         .provider()
    //         .get_transaction_receipt(deployment_result.transaction_hash)
    //         .await
    //     {
    //         Ok(receipt) => {
    //             break receipt;
    //         }
    //         Err(e) => {
    //             eprintln!("Error fetching receipt: {}, retrying...", e);
    //         }
    //     }
    //     tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    // };

    // println!("\nDeployment receipt: {:?}", deployment_receipt);

    // let deployed_contract_address = match &deployment_receipt {
    //     TxnReceipt::Deploy(receipt) => receipt.contract_address,
    //     TxnReceipt::Invoke(receipt) => {
    //         println!("invoke txn receipt{:?}", receipt);
    //         if let Some(contract_address) = receipt
    //             .common_receipt_properties
    //             .events
    //             .first()
    //             .and_then(|event| {
    //                 println!("Processing event: {:?}", event);
    //                 event.data.first()
    //             })
    //         {
    //             println!("Extracted contract address: {:?}", contract_address);
    //             *contract_address
    //         } else {
    //             panic!("No contract address found in the events!");
    //         }
    //     }
    //     _ => {
    //         panic!("wut");
    //     }
    // };

    // println!(
    //     "\nDeployed Contract Address {:?}\n",
    //     deployed_contract_address
    // );

    // println!(
    //     "\nAfter Deploy V3 --> Nonces: {:?}\n",
    //     get_nonces(random_account.clone()).await
    // );

    // // --> INVOKE V3 - nonce : n0 + 2

    // let increase_balance_call = Call {
    //     to: deployed_contract_address,
    //     selector: get_selector_from_name("increase_balance").unwrap(),
    //     calldata: vec![Felt::from_hex("0x50").unwrap()],
    // };

    // let invoke_increase_balance_result = random_account
    //     .execute_v3(vec![increase_balance_call.clone()])
    //     .send()
    //     .await
    //     .unwrap();

    // wait_for_sent_transaction(
    //     invoke_increase_balance_result.transaction_hash,
    //     &random_account.random_accounts().unwrap(),
    // )
    // .await
    // .unwrap();

    // println!(
    //     "After Invoke V3 --> Nonces: {:?}",
    //     get_nonces(random_account.clone()).await
    // );

    // // --> INVOKE V3

    // let invoke_increase_balance_result = random_account
    //     .execute_v3(vec![increase_balance_call])
    //     .send()
    //     .await
    //     .unwrap();

    // wait_for_sent_transaction(
    //     invoke_increase_balance_result.transaction_hash,
    //     &random_account.random_accounts().unwrap(),
    // )
    // .await
    // .unwrap();

    // println!(
    //     "After Invoke V3 --> Nonces: {:?}",
    //     get_nonces(random_account).await
    // );
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct GetNoncesResponse {
    latest_nonce: Felt,
    pending_nonce: Felt,
    account_nonce: Felt,
}

async fn get_nonces(random_account: RandomSingleOwnerAccount) -> GetNoncesResponse {
    let latest_nonce = random_account
        .provider()
        .get_nonce(BlockId::Tag(BlockTag::Latest), random_account.address())
        .await
        .unwrap();

    let pending_nonce = random_account
        .provider()
        .get_nonce(BlockId::Tag(BlockTag::Latest), random_account.address())
        .await
        .unwrap();

    let account_nonce = random_account.get_nonce().await.unwrap();

    GetNoncesResponse {
        latest_nonce,
        pending_nonce,
        account_nonce,
    }
}
