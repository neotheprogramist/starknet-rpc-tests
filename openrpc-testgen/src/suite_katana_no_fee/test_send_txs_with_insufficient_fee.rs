use crate::{
    assert_eq_result, assert_matches_result,
    utils::v7::{
        accounts::{
            account::{Account, ConnectedAccount},
            call::Call,
        },
        endpoints::{
            errors::OpenRpcTestGenError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use starknet_types_core::felt::Felt;

pub const DEFAULT_PREFUNDED_ACCOUNT_BALANCE: u128 = 10 * u128::pow(10, 21);

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatanaNoFee;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;

        let initial_nonce = account.get_nonce().await?;

        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        // -----------------------------------------------------------------------
        //  transaction with low max fee (underpriced).

        let res = account
            .execute_v1(vec![increase_balance_call.clone()])
            .max_fee(Felt::TWO)
            .send()
            .await;

        // In no fee mode, the transaction resources (ie max fee) is totally ignored. So doesn't
        // matter what value is set, the transaction will always be executed successfully.
        assert_matches_result!(res, Ok(tx) => {
            let tx_hash = tx.transaction_hash;
            assert_matches_result!(wait_for_sent_transaction(tx_hash, &account).await, Ok(_));
        });

        let nonce = account.get_nonce().await?;
        assert_eq_result!(
            initial_nonce + 1,
            nonce,
            "Nonce should change in fee-disabled mode"
        );

        // -----------------------------------------------------------------------
        //  transaction with insufficient balance.

        let fee = Felt::from(DEFAULT_PREFUNDED_ACCOUNT_BALANCE + 1);

        let res = account
            .execute_v1(vec![increase_balance_call])
            .max_fee(fee)
            .send()
            .await;

        // in no fee mode, account balance is ignored. as long as the max fee (aka resources) is
        // enough to at least run the account validation, the tx should be accepted.
        // Wait for the transaction to be accepted
        wait_for_sent_transaction(res?.transaction_hash, &account).await?;

        // nonce should be incremented by 1 after a valid tx.
        let nonce = account.get_nonce().await?;
        assert_eq_result!(
            initial_nonce + 2,
            nonce,
            "Nonce should change in fee-disabled mode"
        );

        Ok(Self {})
    }
}
