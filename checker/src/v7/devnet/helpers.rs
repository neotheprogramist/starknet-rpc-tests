use super::endpoints::postman_load_l1_messaging_contract;
use super::errors::DevnetError;
use super::models::PostmanLoadL1MessagingContractParams;
use crate::v7::devnet::models::MsgToL2;

use openrpc_checker::v7::accounts::account::{Account, AccountError};
use openrpc_checker::v7::accounts::creation::create::{create_account, AccountType};
use openrpc_checker::v7::accounts::creation::helpers::get_chain_id;
use openrpc_checker::v7::accounts::deployment::deploy::deploy_account;
use openrpc_checker::v7::accounts::deployment::structs::{ValidatedWaitParams, WaitForTx};
use openrpc_checker::v7::accounts::single_owner::{ExecutionEncoding, SingleOwnerAccount};
use openrpc_checker::v7::contract::factory::ContractFactory;
use openrpc_checker::v7::endpoints::declare_contract::{
    extract_class_hash_from_error, parse_class_hash_from_error, RunnerError,
};
use openrpc_checker::v7::endpoints::errors::{CallError, RpcError};
use openrpc_checker::v7::endpoints::utils::{
    get_compiled_contract, get_selector_from_name, setup_generated_account, validate_inputs,
    wait_for_sent_transaction,
};
use openrpc_checker::v7::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use openrpc_checker::v7::providers::provider::{Provider, ProviderError};
use openrpc_checker::v7::signers::key_pair::SigningKey;
use openrpc_checker::v7::signers::local_wallet::LocalWallet;
use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::v0_7_1::{BlockId, BlockTag, MsgFromL1};
use starknet_types_rpc::TxnReceipt;
use std::str::FromStr;
use tracing::info;
use url::Url;

#[allow(clippy::too_many_arguments)]
pub async fn prepare_postman_send_message_to_l2(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    l1_url: Url,
    class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<MsgToL2<Felt>, DevnetError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(RpcError::CreationError(e).into());
            }
        };

    let (
        account_address,
        private_key,
        erc20_strk_contract_address,
        erc20_eth_contract_address,
        amount_per_test,
    ) = validate_inputs(
        account_address,
        private_key,
        erc20_strk_contract_address,
        erc20_eth_contract_address,
        amount_per_test,
    )?;

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account.clone(),
        erc20_eth_contract_address,
        erc20_strk_contract_address,
        amount_per_test,
        create_acc_data.address,
    )
    .await?;

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let result = match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => value,
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(RpcError::from(e).into());
        }
    };

    wait_for_sent_transaction(result, &user_passed_account).await?;

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        JsonRpcClient::new(HttpTransport::new(url.clone())),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let hash = match account
        .declare_v3(flattened_sierra_class, compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string())?)
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string())?)
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    starkneterror
                ))))
            }
        }
        Err(e) => {
            let full_error_message = format!("{:?}", e);
            Ok(extract_class_hash_from_error(&full_error_message).unwrap())
        }
    };

    let txhash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            let result = factory
                .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .send()
                .await
                .unwrap();
            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    };

    let receipt = provider
        .get_transaction_receipt(txhash.unwrap().transaction_hash)
        .await
        .unwrap();

    let contract_address = match receipt {
        TxnReceipt::Deploy(receipt) => receipt.contract_address,
        TxnReceipt::Invoke(receipt) => {
            if let Some(contract_address) = receipt
                .common_receipt_properties
                .events
                .first()
                .and_then(|event| event.data.first())
            {
                *contract_address
            } else {
                info!("No contract address in Event");
                Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
            }
        }
        _ => {
            info!("Unexpected response type TxnReceipt {:?}", receipt);
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let estimate = JsonRpcClient::new(HttpTransport::new(url.clone()))
        .estimate_message_fee(
            MsgFromL1 {
                from_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                to_address: contract_address,
                entry_point_selector: get_selector_from_name("deposit")?,
                payload: vec![(1_u32).into(), (10_u32).into()],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;

    let l1_contract_address = postman_load_l1_messaging_contract(
        url,
        PostmanLoadL1MessagingContractParams {
            network_url: l1_url.to_string(),
            address: Some("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()),
        },
    )
    .await?;

    let msg_to_l2 = MsgToL2 {
        l2_contract_address: contract_address,
        entry_point_selector: get_selector_from_name("deposit")?,
        l1_contract_address: Felt::from_str(&l1_contract_address.messaging_contract_address)?,
        payload: vec![(1_u32).into(), (10_u32).into()],
        paid_fee_on_l1: estimate.overall_fee,
        nonce: Felt::ZERO,
    };
    Ok(msg_to_l2)
}
