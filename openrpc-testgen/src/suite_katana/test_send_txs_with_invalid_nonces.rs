use crate::{
    assert_eq_result, assert_matches_result,
    utils::v7::{
        accounts::{
            account::{Account, AccountError, ConnectedAccount},
            call::Call,
        },
        endpoints::{
            errors::OpenRpcTestGenError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        providers::{jsonrpc::StarknetError, provider::ProviderError},
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use starknet_types_core::felt::Felt;

pub const DEFAULT_PREFUNDED_ACCOUNT_BALANCE: u128 = 10 * u128::pow(10, 21);

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;

        // set the fee manually here to skip fee estimation. we want to test the pool validator.
        let fee = Felt::from_hex_unchecked("0x1111111111111"); // Max fee 0x1111111111 too low with 0x1111111111111 working correctly.

        let initial_nonce = account.get_nonce().await?;

        // send a valid transaction first to increment the nonce (so that we can test nonce < current
        // nonce later)
        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        let res = account
            .execute_v1(vec![increase_balance_call.clone()])
            .max_fee(fee)
            .send()
            .await?;

        wait_for_sent_transaction(res.transaction_hash, &account).await?;

        // initial sender's account nonce. use to assert how the txs validity change the account nonce.
        let valid_nonce = account.get_nonce().await?;
        assert_eq_result!(
            initial_nonce + 1,
            valid_nonce,
            "Initial nonce after sending tx should be greater by 1."
        );

        // -----------------------------------------------------------------------
        //  transaction with nonce < account nonce.
        let old_nonce = valid_nonce - Felt::ONE;

        let res = account
            .execute_v1(vec![increase_balance_call.clone()])
            .max_fee(fee)
            .nonce(old_nonce)
            .send()
            .await;

        assert_matches_result!(
            res.unwrap_err(),
            AccountError::Provider(ProviderError::StarknetError(
                StarknetError::InvalidTransactionNonce
            ))
        );

        let nonce = account.get_nonce().await?;
        assert_eq_result!(valid_nonce, nonce, "Nonce shouldn't change on invalid tx.");

        // -----------------------------------------------------------------------
        //  transaction with nonce = account nonce.

        let curr_nonce = valid_nonce;

        let res = account
            .execute_v1(vec![increase_balance_call.clone()])
            .max_fee(fee)
            .nonce(curr_nonce)
            .send()
            .await?;

        wait_for_sent_transaction(res.transaction_hash, &account).await?;

        let nonce = account.get_nonce().await?;
        assert_eq!(
            nonce,
            initial_nonce + 2,
            "Nonce should be greater by 2 than initial nonce after sending two valid txs."
        );

        // -----------------------------------------------------------------------
        //  transaction with nonce >= account nonce.
        //
        // ideally, tx with nonce >= account nonce should be considered as valid BUT not to be executed
        // immediately and should be kept around in the pool until the nonce is reached. however,
        // katana doesn't support this feature yet so the current behaviour is to treat the tx as
        // invalid with nonce mismatch error.
        let new_nonce = Felt::from_hex_unchecked("0x100");

        let res = account
            .execute_v1(vec![increase_balance_call.clone()])
            .max_fee(fee)
            .nonce(new_nonce)
            .send()
            .await;

        assert_matches_result!(
            res.unwrap_err(),
            AccountError::Provider(ProviderError::StarknetError(
                StarknetError::InvalidTransactionNonce
            ))
        );

        let nonce = account.get_nonce().await?;
        assert_eq_result!(
            nonce,
            initial_nonce + 2,
            "Nonce shouldn't change bcs the tx is still invalid."
        );

        Ok(Self {})
    }
}
