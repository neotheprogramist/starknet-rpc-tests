use std::sync::Arc;

use rand::{rngs::StdRng, RngCore, SeedableRng};

use starknet_types_core::felt::Felt;
use starknet_types_rpc::{
    v0_7_1::{
        AddInvokeTransactionResult, BlockId, BlockTag, BlockWithTxHashes, BlockWithTxs,
        ContractClass, DeployAccountTxn, DeployAccountTxnV3, FeeEstimate, FunctionCall, InvokeTxn,
        InvokeTxnV1, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs,
        MaybePendingStateUpdate, StateUpdate, Txn, TxnExecutionStatus, TxnReceipt, TxnStatus,
    },
    DeclareTxn, DeployTxn, InvokeTxnReceipt, MsgFromL1,
};

use tracing::{info, warn};
use url::Url;

use crate::v7::rpc::{
    accounts::{
        account::{Account, AccountError, ConnectedAccount},
        call::Call,
        creation::{
            create::{create_account, AccountType},
            helpers::get_chain_id,
        },
        deployment::{
            deploy::deploy_account,
            structs::{ValidatedWaitParams, WaitForTx},
        },
        single_owner::{ExecutionEncoding, SingleOwnerAccount},
    },
    contract::factory::ContractFactory,
    endpoints::{declare_contract::extract_class_hash_from_error, errors::CallError},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient, StarknetError},
        provider::{Provider, ProviderError},
    },
    signers::{key_pair::SigningKey, local_wallet::LocalWallet},
};

use super::{
    declare_contract::{parse_class_hash_from_error, RunnerError},
    errors::RpcError,
    utils::{
        get_compiled_contract, get_selector_from_name, setup_generated_account, validate_inputs,
        wait_for_sent_transaction,
    },
};

#[allow(clippy::too_many_arguments)]
pub async fn add_declare_transaction_v2(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<Felt, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                warn!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;
    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider,
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            info!("General account error encountered: {:?}, possible cause - incorrect address or public_key in environment variables!", e);
            Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                "Account error: {}",
                e
            ))))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn add_declare_transaction_v3(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<Felt, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                warn!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    match account
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
            info!("General account error encountered: {:?}, possible cause - incorrect address or public_key in environment variables!", e);
            Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                "Account error: {}",
                e
            ))))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn add_invoke_transaction_v1(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account);
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn add_invoke_transaction_v3(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account);
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .send()
                .await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn invoke_contract_v1(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declaration_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    let deployment_hash = match declaration_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;

            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;
            Ok(result.transaction_hash)
        }
        Err(e) => {
            info!("Could not deploy the contract: {}", e);
            Err(e)
        }
    };

    let deployment_receipt = match deployment_hash {
        Ok(hash) => provider.get_transaction_receipt(hash).await?,
        Err(e) => {
            info!("Failed to get transaction hash for txn receipt: {}", e);
            return Err(e);
        }
    };

    let contract_address = match deployment_receipt {
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
            info!(
                "Unexpected response type TxnReceipt {:?}",
                deployment_receipt
            );
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let call = Call {
        to: contract_address,
        selector: get_selector_from_name("increase_balance").unwrap(),
        calldata: vec![Felt::from_hex_unchecked("0x50")],
    };

    let invoke_contract_fn_result = account.execute_v1(vec![call]).send().await.unwrap();
    Ok(invoke_contract_fn_result)
}

#[allow(clippy::too_many_arguments)]
pub async fn invoke_contract_v3(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<AddInvokeTransactionResult<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };

    let txhash: Result<AddInvokeTransactionResult<Felt>, RpcError> = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            let result = factory
                .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .send()
                .await?;
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
        .await?;

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

    let call = Call {
        to: contract_address,
        selector: get_selector_from_name("increase_balance").unwrap(),
        calldata: vec![Felt::from_hex_unchecked("0x50")],
    };

    let call_contract_fn_result = account.execute_v3(vec![call]).send().await.unwrap();
    Ok(call_contract_fn_result)
}

pub async fn block_number(url: Url) -> Result<u64, RpcError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    match rpc_client.block_number().await {
        Ok(block_number) => Ok(block_number),
        Err(e) => Err(RpcError::ProviderError(e)),
    }
}

pub async fn chain_id(url: Url) -> Result<Felt, RpcError> {
    let rpc_client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    match rpc_client.chain_id().await {
        Ok(chain_id) => Ok(chain_id),
        Err(e) => Err(RpcError::ProviderError(e)),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn call(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<Vec<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    let deply_contract_hash = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;
            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    };

    let receipt = provider
        .get_transaction_receipt(deply_contract_hash.unwrap().transaction_hash)
        .await?;

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

    let balance = provider
        .call(
            FunctionCall {
                calldata: vec![],
                contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
            },
            BlockId::Tag(BlockTag::Pending),
        )
        .await?;

    Ok(balance)
}

#[allow(clippy::too_many_arguments)]
pub async fn estimate_message_fee(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<FeeEstimate<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    let deply_contract_hash = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;
            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    };

    let receipt = provider
        .get_transaction_receipt(deply_contract_hash.unwrap().transaction_hash)
        .await?;

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

    let estimate = provider
        .estimate_message_fee(
            MsgFromL1 {
                from_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                to_address: contract_address,
                entry_point_selector: get_selector_from_name("deposit").unwrap(),
                payload: vec![(1_u32).into(), (10_u32).into()],
            },
            BlockId::Tag(BlockTag::Pending),
        )
        .await?;

    Ok(estimate)
}

pub async fn get_block_transaction_count(url: Url) -> Result<u64, RpcError> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let count = client
        .get_block_transaction_count(BlockId::Tag(BlockTag::Latest))
        .await?;
    Ok(count)
}

pub async fn get_block_with_tx_hashes(url: Url) -> Result<BlockWithTxHashes<Felt>, RpcError> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let block = client
        .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
        .await?;

    let response = match block {
        MaybePendingBlockWithTxHashes::Block(block) => block,
        _ => {
            panic!("unexpected block response type")
        }
    };
    Ok(response)
}

pub async fn get_block_with_txs(url: Url) -> Result<BlockWithTxs<Felt>, RpcError> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let block = client
        .get_block_with_txs(BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap();

    let block = match block {
        MaybePendingBlockWithTxs::Block(block) => block,
        _ => panic!("unexpected block response type"),
    };

    Ok(block)
}

pub async fn get_state_update(url: Url) -> Result<StateUpdate<Felt>, RpcError> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let state: MaybePendingStateUpdate<Felt> = client
        .get_state_update(BlockId::Tag(BlockTag::Latest))
        .await
        .unwrap();

    let state = match state {
        MaybePendingStateUpdate::Block(state) => state,
        _ => panic!("unexpected block response type"),
    };

    Ok(state)
}

pub async fn get_storage_at(
    url: Url,
    erc20_eth_contract_address: Option<Felt>,
) -> Result<Felt, RpcError> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let erc20_eth_address = match erc20_eth_contract_address {
        Some(address) => address,
        None => Felt::from_hex("049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")?,
    };
    let key: Felt =
        Felt::from_hex("0000000000000000000000000000000000000000000000000000000000000001")?;
    // Checks L2 ETH balance via storage taking advantage of implementation detail
    let storage_value = client
        .get_storage_at(erc20_eth_address, key, BlockId::Tag(BlockTag::Latest))
        .await?;
    Ok(storage_value)
}

#[allow(clippy::too_many_arguments)]
pub async fn get_transaction_status_succeeded(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<TxnStatus, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    let deply_contract_hash = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;
            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    };

    let receipt = provider
        .get_transaction_receipt(deply_contract_hash.unwrap().transaction_hash)
        .await?;

    let tx_hash = match receipt {
        TxnReceipt::Deploy(receipt) => receipt.common_receipt_properties.transaction_hash,
        TxnReceipt::Invoke(receipt) => receipt.common_receipt_properties.transaction_hash,
        _ => {
            info!("Unexpected response type TxnReceipt {:?}", receipt);
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let status = account
        .provider()
        .get_transaction_status(tx_hash)
        .await
        .unwrap();
    match status.finality_status {
        TxnStatus::AcceptedOnL2 => match status.execution_status {
            Some(TxnExecutionStatus::Succeeded) => Ok(TxnStatus::AcceptedOnL2),
            Some(TxnExecutionStatus::Reverted) => Err(RpcError::TxnExecutionStatus(
                "Execution reverted".to_string(),
            )),
            None => Err(RpcError::TxnExecutionStatus(
                "Execution status is None".to_string(),
            )),
        },
        _ => panic!("unexpected transaction status"),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_transaction_by_hash_invoke(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<InvokeTxnV1<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };

    let transaction_hash = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;
            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;

            result.transaction_hash
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let txn = account
        .provider()
        .get_transaction_by_hash(transaction_hash)
        .await
        .unwrap();

    let txn = match txn {
        Txn::Invoke(InvokeTxn::V1(tx)) => tx,
        _ => panic!("unexpected tx response type"),
    };

    Ok(txn)
}

pub async fn get_transaction_by_hash_deploy_acc(
    url: Url,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<DeployAccountTxnV3<Felt>, RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

    let txn = provider
        .get_transaction_by_hash(deploy_account_txn_hash)
        .await
        .unwrap();

    let txn = match txn {
        Txn::DeployAccount(DeployAccountTxn::V3(tx)) => tx,
        _ => panic!("unexpected tx response type"),
    };

    Ok(txn)
}

pub async fn get_transaction_by_block_id_and_index(
    url: Url,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<Txn<Felt>, RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

    let block = provider.block_hash_and_number().await?;

    let block_txn_count = provider
        .get_block_transaction_count(BlockId::Hash(block.block_hash))
        .await?;

    let txn = provider
        .get_transaction_by_block_id_and_index(BlockId::Hash(block.block_hash), block_txn_count - 1)
        .await?;

    let txn = match txn {
        Txn::Invoke(InvokeTxn::V0(txn)) => Txn::Invoke(InvokeTxn::V0(txn)),
        Txn::Invoke(InvokeTxn::V1(txn)) => Txn::Invoke(InvokeTxn::V1(txn)),
        Txn::Invoke(InvokeTxn::V3(txn)) => Txn::Invoke(InvokeTxn::V3(txn)),
        Txn::Declare(DeclareTxn::V0(txn)) => Txn::Declare(DeclareTxn::V0(txn)),
        Txn::Declare(DeclareTxn::V1(txn)) => Txn::Declare(DeclareTxn::V1(txn)),
        Txn::Declare(DeclareTxn::V2(txn)) => Txn::Declare(DeclareTxn::V2(txn)),
        Txn::Declare(DeclareTxn::V3(txn)) => Txn::Declare(DeclareTxn::V3(txn)),
        Txn::DeployAccount(DeployAccountTxn::V1(txn)) => {
            Txn::DeployAccount(DeployAccountTxn::V1(txn))
        }
        Txn::DeployAccount(DeployAccountTxn::V3(txn)) => {
            Txn::DeployAccount(DeployAccountTxn::V3(txn))
        }
        Txn::Deploy(DeployTxn {
            class_hash,
            constructor_calldata,
            contract_address_salt,
            version,
        }) => Txn::Deploy(DeployTxn {
            class_hash,
            constructor_calldata,
            contract_address_salt,
            version,
        }),
        _ => {
            let error_message = format!("Unexpected transaction response type: {:?}", txn);
            return Err(RpcError::UnexpectedTxnType(error_message));
        }
    };

    Ok(txn)
}

pub async fn get_transaction_by_hash_non_existent_tx(url: Url) -> Result<(), RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let err = provider
        .get_transaction_by_hash(Felt::from_hex("0xdeafbeefdeadbeef").unwrap())
        .await
        .unwrap_err();

    match err {
        ProviderError::StarknetError(StarknetError::TransactionHashNotFound) => Ok(()),
        _ => panic!("Unexpected error"),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_transaction_receipt(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<InvokeTxnReceipt<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };

    let txhash: Result<AddInvokeTransactionResult<Felt>, RpcError> = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            let result = factory
                .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .send()
                .await?;
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
        .await?;

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

    let call = Call {
        to: contract_address,
        selector: get_selector_from_name("increase_balance").unwrap(),
        calldata: vec![Felt::from_hex_unchecked("0x50")],
    };

    let result = account.execute_v3(vec![call]).send().await.unwrap();
    wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;

    let receipt = provider
        .get_transaction_receipt(result.transaction_hash)
        .await
        .unwrap();

    match receipt {
        TxnReceipt::Invoke(receipt) => Ok(receipt),
        _ => Err(RpcError::CallError(CallError::UnexpectedReceiptType))?,
    }
}

// #[allow(dead_code)]
// pub async fn get_transaction_receipt_revert(
//     url: Url,
//     sierra_path: &str,
//     casm_path: &str,
//     account_class_hash: Option<Felt>,
//     account_address: Option<Felt>,
//     private_key: Option<Felt>,
//     erc20_strk_contract_address: Option<Felt>,
//     erc20_eth_contract_address: Option<Felt>,
//     amount_per_test: Option<Felt>,
// ) -> Result<(), RpcError> {
//     let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
//     let create_acc_data =
//         match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
//             Ok(value) => value,
//             Err(e) => {
//                 info!("{}", "Could not create an account");
//                 return Err(e.into());
//             }
//         };

//     let (
//         account_address,
//         private_key,
//         erc20_strk_contract_address,
//         erc20_eth_contract_address,
//         amount_per_test,
//     ) = validate_inputs(
//         account_address,
//         private_key,
//         erc20_strk_contract_address,
//         erc20_eth_contract_address,
//         amount_per_test,
//     )?;

//     let chain_id = get_chain_id(&provider).await?;

//     let user_passed_account = SingleOwnerAccount::new(
//         provider.clone(),
//         LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
//         account_address,
//         chain_id,
//         ExecutionEncoding::New,
//     );

//     setup_generated_account(
//         user_passed_account,
//         erc20_eth_contract_address,
//         erc20_strk_contract_address,
//         amount_per_test,
//         create_acc_data.address,
//     )
//     .await?;

//     let wait_conifg = WaitForTx {
//         wait: true,
//         wait_params: ValidatedWaitParams::default(),
//     };

//     match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
//         Ok(value) => Some(value),
//         Err(e) => {
//             info!("{}", "Could not deploy an account");
//             return Err(e.into());
//         }
//     };

//     let sender_address = create_acc_data.address;
//     let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

//     let mut account = SingleOwnerAccount::new(
//         provider.clone(),
//         signer,
//         sender_address,
//         chain_id,
//         ExecutionEncoding::New,
//     );

//     account.set_block_id(BlockId::Tag(BlockTag::Pending));
//     let transfer_execution = account
//         .execute_v1(vec![Call {
//             to: erc20_eth_contract_address,
//             selector: get_selector_from_name("transfer")?,
//             calldata: vec![account_address, amount_per_test, Felt::ZERO],
//         }])
//         .send()
//         .await
//         .unwrap();
//     info!("ok");

//     let receipt = account
//         .provider()
//         .get_transaction_receipt(transfer_execution.transaction_hash)
//         .await
//         .unwrap();

//     match receipt {
//         TxnReceipt::Invoke(invoke_receipt) => match invoke_receipt.common_receipt_properties.anon {
//             Anonymous::Reverted(_) => {
//                 info!("reverted");
//                 Ok(())
//             }
//             Anonymous::Successful(_) => {
//                 info!("successful");
//                 Err(RpcError::CallError(CallError::UnexpectedExecutionResult))
//             }
//             _ => {
//                 info!("other");
//                 Err(RpcError::CallError(CallError::UnexpectedExecutionResult))
//             }
//         },
//         _ => {
//             info!("Unexpected response type TxnReceipt: {:?}", receipt);
//             Err(RpcError::CallError(CallError::UnexpectedReceiptType))
//         }
//     }
// }

#[allow(clippy::too_many_arguments)]
pub async fn get_class(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<ContractClass<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };

    let contract_class = account
        .provider()
        .get_class(BlockId::Tag(BlockTag::Latest), declare_contract_hash?)
        .await
        .unwrap();

    Ok(contract_class)
}

#[allow(clippy::too_many_arguments)]
pub async fn get_class_hash_at(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<Felt, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    let deply_contract_hash = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;
            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    };

    let receipt = provider
        .get_transaction_receipt(deply_contract_hash.unwrap().transaction_hash)
        .await?;

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
    let contract_class_hash = account
        .provider()
        .get_class_hash_at(BlockId::Tag(BlockTag::Pending), contract_address)
        .await
        .unwrap();

    Ok(contract_class_hash)
}

#[allow(clippy::too_many_arguments)]
pub async fn get_class_at(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
    account_class_hash: Option<Felt>,
    account_address: Option<Felt>,
    private_key: Option<Felt>,
    erc20_strk_contract_address: Option<Felt>,
    erc20_eth_contract_address: Option<Felt>,
    amount_per_test: Option<Felt>,
) -> Result<ContractClass<Felt>, RpcError> {
    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await?;

    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, account_class_hash).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
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

    let chain_id = get_chain_id(&provider).await?;

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

    let deploy_account_txn_hash =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    wait_for_sent_transaction(deploy_account_txn_hash, &user_passed_account).await?;

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

    let declare_contract_hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
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
            Ok(extract_class_hash_from_error(&full_error_message)?)
        }
    };
    let deply_contract_hash = match declare_contract_hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000")?)
                .send()
                .await?;
            wait_for_sent_transaction(result.transaction_hash, &user_passed_account).await?;
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    };

    let receipt = provider
        .get_transaction_receipt(deply_contract_hash.unwrap().transaction_hash)
        .await?;

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

    let contract_class = account
        .provider()
        .get_class_at(BlockId::Tag(BlockTag::Pending), contract_address)
        .await?;

    Ok(contract_class)
}
