use std::path::PathBuf;

use rand::{seq::SliceRandom, thread_rng, Rng};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag};
use url::Url;

use crate::{
    utils::v7::{
        accounts::{
            account::{Account, AccountError, ConnectedAccount, ExecutionEncoder},
            call::Call,
            creation::{
                create::{create_account, AccountType},
                helpers::get_chain_id,
            },
            single_owner::{ExecutionEncoding, SignError, SingleOwnerAccount},
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
    RandomizableAccountsTrait, SetupableTrait,
};
pub mod suite_deploy;
pub mod test_declare_txn_v2;
pub mod test_declare_txn_v3;

pub struct TestSuiteOpenRpc {
    pub urls: Vec<Url>,
    pub paymaster_account_address: Felt,
    pub paymaster_private_key: Felt,
    pub udc_address: Felt,
    pub executable_account_sierra_path: PathBuf,
    pub executable_account_casm_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct SetupOutput {
    pub random_paymaster_account: RandomSingleOwnerAccount,
    pub random_executable_account: RandomSingleOwnerAccount,
}

#[derive(Clone, Debug)]
pub struct RandomSingleOwnerAccount {
    pub accounts: Vec<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
}

impl RandomizableAccountsTrait for RandomSingleOwnerAccount {
    fn random_accounts(
        &self,
    ) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, RpcError> {
        let mut rng = thread_rng();
        let account = self.accounts.choose(&mut rng).cloned().ok_or_else(|| {
            RpcError::EmptyUrlList("Accounts list is empty - no urls.".to_string())
        })?;
        Ok(account)
    }
}

impl RandomSingleOwnerAccount {
    pub fn new(
        accounts: Vec<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
    ) -> Self {
        Self { accounts }
    }

    pub fn set_block_id(
        &mut self,
        block_id: BlockId<Felt>,
    ) -> SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet> {
        let mut acc = self.random_accounts().unwrap();
        acc.set_block_id(block_id).clone()
    }
}

impl Account for RandomSingleOwnerAccount {
    type SignError =
        SignError<<LocalWallet as crate::utils::v7::signers::signer::Signer>::SignError>;

    fn address(&self) -> Felt {
        let random_account = self.random_accounts().unwrap();
        random_account.address()
    }

    fn chain_id(&self) -> Felt {
        let random_account = self.random_accounts().unwrap();
        random_account.chain_id()
    }

    async fn sign_execution_v1(
        &self,
        execution: &crate::utils::v7::accounts::account::RawExecutionV1,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        let random_account = self.random_accounts().unwrap();
        random_account
            .sign_execution_v1(execution, query_only)
            .await
    }

    async fn sign_execution_v3(
        &self,
        execution: &crate::utils::v7::accounts::account::RawExecutionV3,
        _query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        let random_account = self.random_accounts().unwrap();
        random_account
            .sign_execution_v3(execution, _query_only)
            .await
    }

    async fn sign_declaration_v2(
        &self,
        declaration: &crate::utils::v7::accounts::account::RawDeclarationV2,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        let random_account = self.random_accounts().unwrap();
        random_account
            .sign_declaration_v2(declaration, query_only)
            .await
    }

    async fn sign_declaration_v3(
        &self,
        declaration: &crate::utils::v7::accounts::account::RawDeclarationV3,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        let random_account = self.random_accounts().unwrap();
        random_account
            .sign_declaration_v3(declaration, query_only)
            .await
    }

    fn is_signer_interactive(&self) -> bool {
        let random_account = self.random_accounts().unwrap();
        random_account.is_signer_interactive()
    }
}

impl ExecutionEncoder for RandomSingleOwnerAccount {
    fn encode_calls(&self, calls: &[Call]) -> Vec<Felt> {
        let random_account = self.random_accounts().unwrap();
        random_account.encode_calls(calls)
    }
}

impl ConnectedAccount for RandomSingleOwnerAccount {
    type Provider = JsonRpcClient<HttpTransport>;

    fn provider(&self) -> &Self::Provider {
        let mut rng = thread_rng();
        self.accounts[rng.gen_range(0..self.accounts.len())].provider()
    }

    fn block_id(&self) -> BlockId<Felt> {
        let random_account = self.random_accounts().unwrap();
        random_account.block_id()
    }

    async fn get_nonce(&self) -> Result<Felt, ProviderError> {
        let random_account = self.random_accounts().unwrap();
        random_account.get_nonce().await
    }
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

        let provider = JsonRpcClient::new(HttpTransport::new(self.urls[0].clone()));
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
            provider.clone(),
            LocalWallet::from(executable_account_data.signing_key),
            executable_account_data.address,
            chain_id,
            ExecutionEncoding::New,
        );

        executable_account.set_block_id(BlockId::Tag(BlockTag::Pending));

        let mut paymaster_accounts = vec![];
        let mut executable_accounts = vec![];
        for url in &self.urls {
            let provider = JsonRpcClient::new(HttpTransport::new(url.clone()));
            let chain_id = get_chain_id(&provider).await?;

            let paymaster_account = SingleOwnerAccount::new(
                provider.clone(),
                LocalWallet::from(paymaster_signing_key),
                self.paymaster_account_address,
                chain_id,
                ExecutionEncoding::New,
            );

            let executable_account = SingleOwnerAccount::new(
                provider.clone(),
                LocalWallet::from(executable_account_data.signing_key),
                executable_account_data.address,
                chain_id,
                ExecutionEncoding::New,
            );

            paymaster_accounts.push(paymaster_account);
            executable_accounts.push(executable_account);
        }
        // change name to be less confusing
        Ok(SetupOutput {
            random_executable_account: RandomSingleOwnerAccount {
                accounts: executable_accounts,
            },
            random_paymaster_account: RandomSingleOwnerAccount {
                accounts: paymaster_accounts,
            },
        })
    }
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(
    env!("OUT_DIR"),
    "/generated_tests_suite_openrpc.rs"
));
