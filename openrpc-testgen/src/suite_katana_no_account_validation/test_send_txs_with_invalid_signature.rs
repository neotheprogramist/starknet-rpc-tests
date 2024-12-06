use crate::{
    assert_eq_result,
    utils::v7::{
        accounts::{
            account::{Account, ConnectedAccount},
            call::Call,
            creation::helpers::get_chain_id,
            single_owner::{ExecutionEncoding, SingleOwnerAccount},
        },
        endpoints::{
            errors::OpenRpcTestGenError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        signers::{key_pair::SigningKey, local_wallet::LocalWallet},
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use starknet_types_core::felt::Felt;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatanaNoAccountValidation;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;

        let provider = account.provider().clone();

        let chain_id = get_chain_id(&provider).await?;

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

        wait_for_sent_transaction(res?.transaction_hash, &account_invalid).await?;

        // nonce should be incremented by 1 after a valid tx.
        let nonce = account_invalid.get_nonce().await?;
        assert_eq_result!(initial_nonce + 1, nonce);

        Ok(Self {})
    }
}
