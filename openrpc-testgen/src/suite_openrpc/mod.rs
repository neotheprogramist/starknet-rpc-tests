use std::path::PathBuf;

use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag};
use test_declare_txn_v2::ContractPathPair;
use url::Url;

use crate::{
    utils::v7::{
        accounts::{
            account::{Account, AccountError},
            call::Call,
            creation::{
                create::{create_account, AccountType},
                helpers::get_chain_id,
            },
            single_owner::{ExecutionEncoding, SingleOwnerAccount},
        },
        endpoints::{
            declare_contract::{
                extract_class_hash_from_error, get_compiled_contract, parse_class_hash_from_error,
                RunnerError,
            },
            errors::RpcError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        providers::{
            jsonrpc::{HttpTransport, JsonRpcClient},
            provider::ProviderError,
        },
        signers::{key_pair::SigningKey, local_wallet::LocalWallet},
    },
    SetupableTrait,
};

pub mod test_declare_txn_v2;

pub struct TestSuiteOpenRpc {
    pub url: Url,
    pub paymaster_account_address: Felt,
    pub paymaster_private_key: Felt,
    pub udc_address: Felt,
    pub executable_account_sierra_path: PathBuf,
    pub executable_account_casm_path: PathBuf,
    pub contracts_to_deploy_paths: Vec<ContractPathPair>,
}

#[derive(Clone, Debug)]
pub struct SetupOutput {
    pub paymaster_account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    pub executable_account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    pub contracts_to_deploy_paths: Vec<ContractPathPair>,
}

impl SetupableTrait for TestSuiteOpenRpc {
    type Output = SetupOutput;

    async fn setup(&self) -> Result<Self::Output, RpcError> {
        let (executable_account_flattened_sierra_class, executable_account_compiled_class_hash) =
            get_compiled_contract(
                self.executable_account_sierra_path.clone(),
                self.executable_account_casm_path.clone(),
            )
            .await?;

        let provider = JsonRpcClient::new(HttpTransport::new(self.url.clone()));
        let chain_id = get_chain_id(&provider).await?;

        let paymaster_signing_key = SigningKey::from_secret_scalar(self.paymaster_private_key);
        let paymaster_account = SingleOwnerAccount::new(
            provider.clone(),
            LocalWallet::from(paymaster_signing_key),
            self.paymaster_account_address,
            chain_id,
            ExecutionEncoding::New,
        );

        let declare_executable_account_hash = match paymaster_account
            .declare_v3(
                executable_account_flattened_sierra_class.clone(),
                executable_account_compiled_class_hash,
            )
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
        }?;

        let executable_account_data = create_account(
            &provider,
            AccountType::Oz,
            Option::None,
            Some(declare_executable_account_hash),
        )
        .await?;

        let deploy_executable_account_call: Call = Call {
            to: self.udc_address,
            selector: get_selector_from_name("deployContract")?,
            calldata: vec![
                declare_executable_account_hash,
                executable_account_data.salt,
                Felt::ZERO,
                Felt::ONE,
                SigningKey::verifying_key(&executable_account_data.signing_key).scalar(),
            ],
        };

        let deploy_executable_account_result = paymaster_account
            .execute_v1(vec![deploy_executable_account_call])
            .send()
            .await?;

        wait_for_sent_transaction(
            deploy_executable_account_result.transaction_hash,
            &paymaster_account,
        )
        .await?;

        let mut executable_account = SingleOwnerAccount::new(
            JsonRpcClient::new(HttpTransport::new(self.url.clone())),
            LocalWallet::from(executable_account_data.signing_key),
            executable_account_data.address,
            chain_id,
            ExecutionEncoding::New,
        );

        executable_account.set_block_id(BlockId::Tag(BlockTag::Pending));

        Ok(SetupOutput {
            paymaster_account,
            executable_account,
            contracts_to_deploy_paths: self.contracts_to_deploy_paths.clone(),
        })
    }
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(
    env!("OUT_DIR"),
    "/generated_tests_suite_openrpc.rs"
));
