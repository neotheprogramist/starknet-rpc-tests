use crate::RandomizableAccountsTrait;

use super::v7::{
    accounts::{
        account::{Account, ConnectedAccount, ExecutionEncoder},
        call::Call,
        single_owner::{SignError, SingleOwnerAccount},
    },
    endpoints::errors::OpenRpcTestGenError,
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        provider::ProviderError,
    },
    signers::local_wallet::LocalWallet,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::BlockId;

#[derive(Clone, Debug)]
pub struct RandomSingleOwnerAccount {
    pub accounts: Vec<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
}

impl RandomizableAccountsTrait for RandomSingleOwnerAccount {
    fn random_accounts(
        &self,
    ) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, OpenRpcTestGenError>
    {
        let mut rng = thread_rng();
        let account = self.accounts.choose(&mut rng).cloned().ok_or_else(|| {
            OpenRpcTestGenError::EmptyUrlList("Accounts list is empty - no urls.".to_string())
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
