use std::sync::Arc;

use rand::{rngs::StdRng, RngCore, SeedableRng};

use starknet_types_core::felt::Felt;
use starknet_types_rpc::{
    v0_7_1::{
        AddInvokeTransactionResult, BlockId, BlockTag, BlockWithTxHashes, BlockWithTxs,
        ContractClass, DeployAccountTxn, DeployAccountTxnV3, DeployTxnReceipt, FeeEstimate,
        FunctionCall, InvokeTxn, InvokeTxnV1, MaybePendingBlockWithTxHashes,
        MaybePendingBlockWithTxs, MaybePendingStateUpdate, PriceUnit, StateUpdate, Txn,
        TxnExecutionStatus, TxnReceipt, TxnStatus,
    },
    DeclareTxn, DeployTxn, MsgFromL1,
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
            structs::MintRequest2,
        },
        deployment::{
            deploy::deploy_account,
            structs::{ValidatedWaitParams, WaitForTx},
        },
        single_owner::{ExecutionEncoding, SingleOwnerAccount},
        utils::mint::mint,
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
    },
};

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
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );
    setup_generated_account(
        user_passed_account,
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

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };
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
    // sleep(Duration::from_secs(60)).await;
    match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        JsonRpcClient::new(HttpTransport::new(url.clone())),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    match account
        .declare_v3(flattened_sierra_class, compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };
    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        JsonRpcClient::new(HttpTransport::new(url.clone())),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
    match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account);
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap();
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    }
}

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
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };
    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        JsonRpcClient::new(HttpTransport::new(url.clone())),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let hash = match account
        .declare_v3(flattened_sierra_class, compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
    match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account);
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);
            let result = factory
                .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .send()
                .await
                .unwrap();
            Ok(result)
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            Err(e)
        }
    }
}

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let call = Call {
        to: receipt.contract_address,
        selector: get_selector_from_name("increase_balance").unwrap(),
        calldata: vec![Felt::from_hex_unchecked("0x50")],
    };

    let result = account.execute_v1(vec![call]).send().await.unwrap();

    Ok(result)
}

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v3(flattened_sierra_class, compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let call = Call {
        to: receipt.contract_address,
        selector: get_selector_from_name("increase_balance").unwrap(),
        calldata: vec![Felt::from_hex_unchecked("0x50")],
    };

    let result = account.execute_v3(vec![call]).send().await.unwrap();

    Ok(result)
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let eth_balance = provider
        .call(
            FunctionCall {
                calldata: vec![],
                contract_address: receipt.contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;

    Ok(eth_balance)
}

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(
            Arc::new(flattened_sierra_class.clone()),
            compiled_class_hash,
        )
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let estimate = provider
        .estimate_message_fee(
            MsgFromL1 {
                from_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                to_address: receipt.contract_address,
                entry_point_selector: get_selector_from_name("deposit").unwrap(),
                payload: vec![(1_u32).into(), (10_u32).into()],
            },
            BlockId::Tag(BlockTag::Latest),
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

pub async fn get_storage_at(url: Url) -> Result<Felt, RpcError> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let contract_address =
        Felt::from_hex("049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")?;
    let key: Felt =
        Felt::from_hex("0000000000000000000000000000000000000000000000000000000000000001")?;
    // Checks L2 ETH balance via storage taking advantage of implementation detail
    let eth_balance = client
        .get_storage_at(contract_address, key, BlockId::Tag(BlockTag::Latest))
        .await?;
    Ok(eth_balance)
}

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(
            Arc::new(flattened_sierra_class.clone()),
            compiled_class_hash,
        )
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    let status = account
        .provider()
        .get_transaction_status(receipt.common_receipt_properties.transaction_hash)
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(
            Arc::new(flattened_sierra_class.clone()),
            compiled_class_hash,
        )
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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

    let transaction_hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            let result = factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap();

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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let txn_hash: Felt =
        match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
            Ok(txn_hash) => txn_hash,
            Err(e) => {
                info!("{}", "Could not deploy an account");
                return Err(e.into());
            }
        };

    let txn = provider.get_transaction_by_hash(txn_hash).await.unwrap();

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
    // Change the return type to general Txn
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

    let chain_id = get_chain_id(&provider).await.unwrap();

    let user_passed_account = SingleOwnerAccount::new(
        provider.clone(),
        LocalWallet::from(SigningKey::from_secret_scalar(private_key)),
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    setup_generated_account(
        user_passed_account,
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

    let block = provider.block_hash_and_number().await?;

    let block_txn_count = provider
        .get_block_transaction_count(BlockId::Hash(block.block_hash))
        .await?;

    println!("blockTxnCount {:?}", block_txn_count);

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
        .get_transaction_by_hash(Felt::from_hex("0x55555").unwrap())
        .await
        .unwrap_err();

    match err {
        ProviderError::StarknetError(StarknetError::TransactionHashNotFound) => Ok(()),
        _ => panic!("Unexpected error"),
    }
}

pub async fn get_transaction_receipt(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
) -> Result<DeployTxnReceipt<Felt>, RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, Option::None).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
            }
        };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Fri,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Wei,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v3(flattened_sierra_class, compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
    };

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    //     match receipt.common_receipt_properties. {
    //         TxnExecutionStatus::Succeeded => Ok(receipt),
    //         _ => Err(RpcError::CallError(CallError::UnexpectedExecutionResult))?,
    //     }
    Ok(receipt)
}

#[allow(dead_code)]
pub async fn get_transaction_receipt_revert(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
) -> Result<(), RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, Option::None).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
            }
        };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Fri,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Wei,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
    };

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
                // .max_fee(Felt::from_dec_str("1").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    // match receipt.common_receipt_properties.execution_status {
    //     TxnExecutionStatus::Reverted => Ok(()),
    //     _ => Err(RpcError::CallError(CallError::UnexpectedExecutionResult))?,
    //     TxnExecutionStatus::Succeeded => todo!(),
    // }
    Ok(())
}

pub async fn get_class(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
) -> Result<ContractClass<Felt>, RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, Option::None).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
            }
        };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Fri,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Wei,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
    };

    match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let contract_class = account
        .provider()
        .get_class(BlockId::Tag(BlockTag::Latest), hash.unwrap())
        .await
        .unwrap();

    Ok(contract_class)
}

pub async fn get_class_hash_at(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
) -> Result<Felt, RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, Option::None).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
            }
        };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Fri,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Wei,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
    };

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    // match receipt.common_receipt_properties.finality_status {
    //     TxnExecutionStatus::Succeeded => {}
    //     _ => Err(RpcError::CallError(CallError::UnexpectedExecutionResult))?,
    // }

    let contract_class_hash = account
        .provider()
        .get_class_hash_at(BlockId::Tag(BlockTag::Latest), receipt.contract_address)
        .await
        .unwrap();

    Ok(contract_class_hash)
}

pub async fn get_class_at(
    url: Url,
    sierra_path: &str,
    casm_path: &str,
) -> Result<ContractClass<Felt>, RpcError> {
    let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));

    let create_acc_data =
        match create_account(&provider, AccountType::Oz, Option::None, Option::None).await {
            Ok(value) => value,
            Err(e) => {
                info!("{}", "Could not create an account");
                return Err(e.into());
            }
        };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Fri,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    match mint(
        url.clone(),
        &MintRequest2 {
            amount: u128::MAX,
            address: create_acc_data.address,
            unit: PriceUnit::Wei,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            info!("{}", "Could not mint tokens");
            return Err(e.into());
        }
    };

    let wait_conifg = WaitForTx {
        wait: true,
        wait_params: ValidatedWaitParams::default(),
    };

    let chain_id = get_chain_id(&provider).await.unwrap();

    match deploy_account(&provider, chain_id, wait_conifg, create_acc_data).await {
        Ok(value) => Some(value),
        Err(e) => {
            info!("{}", "Could not deploy an account");
            return Err(e.into());
        }
    };

    let sender_address = create_acc_data.address;
    let signer: LocalWallet = LocalWallet::from(create_acc_data.signing_key);

    let mut account = SingleOwnerAccount::new(
        provider.clone(),
        signer,
        sender_address,
        chain_id,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Latest));

    let (flattened_sierra_class, compiled_class_hash) =
        get_compiled_contract(sierra_path, casm_path).await.unwrap();

    let hash = match account
        .declare_v2(Arc::new(flattened_sierra_class), compiled_class_hash)
        .send()
        .await
    {
        Ok(result) => Ok(result.class_hash),
        Err(AccountError::Signing(sign_error)) => {
            if sign_error.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&sign_error.to_string()))
            } else {
                Err(RpcError::RunnerError(RunnerError::AccountFailure(format!(
                    "Transaction execution error: {}",
                    sign_error
                ))))
            }
        }

        Err(AccountError::Provider(ProviderError::Other(starkneterror))) => {
            if starkneterror.to_string().contains("is already declared") {
                Ok(parse_class_hash_from_error(&starkneterror.to_string()))
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
    };

    let hash = match hash {
        Ok(class_hash) => {
            let factory = ContractFactory::new(class_hash, account.clone());
            let mut salt_buffer = [0u8; 32];
            let mut rng = StdRng::from_entropy();
            rng.fill_bytes(&mut salt_buffer[1..]);

            factory
                .deploy_v1(vec![], Felt::from_bytes_be(&salt_buffer), true)
                .max_fee(Felt::from_dec_str("100000000000000000").unwrap())
                .send()
                .await
                .unwrap()
        }
        Err(e) => {
            info!("Could not deploy the contract {}", e);
            return Err(e);
        }
    };

    let receipt = account
        .provider()
        .get_transaction_receipt(hash.transaction_hash)
        .await
        .unwrap();

    let receipt = match receipt {
        TxnReceipt::Deploy(receipt) => receipt,
        _ => {
            info!("Unexpected response type TxnReceipt");
            Err(RpcError::CallError(CallError::UnexpectedReceiptType))?
        }
    };

    // match receipt.common_receipt_properties.execution_status {
    //     TxnExecutionStatus::Succeeded => {}
    //     _ => Err(RpcError::CallError(CallError::UnexpectedExecutionResult))?,
    // }

    let contract_class = account
        .provider()
        .get_class_at(BlockId::Tag(BlockTag::Latest), receipt.contract_address)
        .await?;

    Ok(contract_class)
}
