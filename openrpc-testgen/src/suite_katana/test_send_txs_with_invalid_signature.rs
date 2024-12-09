use crate::{
    assert_eq_result, assert_matches_result,
    utils::v7::{
        accounts::{
            account::{Account, AccountError, ConnectedAccount},
            call::Call,
            creation::helpers::get_chain_id,
            single_owner::{ExecutionEncoding, SingleOwnerAccount},
        },
        endpoints::{errors::OpenRpcTestGenError, utils::get_selector_from_name},
        providers::{jsonrpc::StarknetError, provider::ProviderError},
        signers::{key_pair::SigningKey, local_wallet::LocalWallet},
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use starknet_types_core::felt::Felt;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;

        let provider = account.provider().clone();

        let chain_id = get_chain_id(&provider).await?;

        // starknet-rs doesn't provide a way to manually set the signatures so instead we create an
        // account with random signer to simulate invalid signatures.
        let account_invalid = SingleOwnerAccount::new(
            account.provider().clone(),
            LocalWallet::from(SigningKey::from_random()),
            account.address(),
            chain_id,
            ExecutionEncoding::New,
        );
        // initial sender's account nonce. use to assert how the txs validity change the account nonce.
        let initial_nonce = account_invalid.get_nonce().await?;

        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        // -----------------------------------------------------------------------
        //  transaction with invalid signatures.

        // we set the max fee manually here to skip fee estimation. we want to test the pool validator.
        let res = account_invalid
            .execute_v1(vec![increase_balance_call])
            .max_fee(Felt::from_hex_unchecked("0x1111111111111")) // Max fee 0x1111111111 too low with 0x1111111111111 working correctly.
            .send()
            .await;

        assert_matches_result!(
            res.unwrap_err(),
            AccountError::Provider(ProviderError::StarknetError(
                StarknetError::ValidationFailure(_)
            ))
        );

        // nonce shouldn't change for an invalid tx.
        let nonce = account_invalid.get_nonce().await?;
        assert_eq_result!(nonce, initial_nonce);

        Ok(Self {})
    }
}
