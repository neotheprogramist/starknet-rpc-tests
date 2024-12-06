use crate::{
    assert_eq_result, assert_matches_result,
    utils::v7::{
        accounts::{
            account::{Account, AccountError, ConnectedAccount},
            call::Call,
        },
        endpoints::{errors::OpenRpcTestGenError, utils::get_selector_from_name},
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

        assert_matches_result!(
            res.unwrap_err(),
            AccountError::Provider(ProviderError::StarknetError(
                StarknetError::InsufficientMaxFee
            ))
        );
        let nonce = account.get_nonce().await?;
        assert_eq_result!(
            nonce,
            initial_nonce,
            "Nonce shouldn't change in fee-enabled mode"
        );

        // -----------------------------------------------------------------------
        //  transaction with insufficient balance.

        let fee = Felt::from(DEFAULT_PREFUNDED_ACCOUNT_BALANCE + 1);

        let res = account
            .execute_v1(vec![increase_balance_call])
            .max_fee(fee)
            .send()
            .await;

        assert_matches_result!(
            res.unwrap_err(),
            AccountError::Provider(ProviderError::StarknetError(
                StarknetError::InsufficientAccountBalance
            ))
        );
        // nonce shouldn't change for an invalid tx.
        let nonce = account.get_nonce().await?;
        assert_eq_result!(
            nonce,
            initial_nonce,
            "Nonce shouldn't change in fee-enabled mode"
        );
        Ok(Self {})
    }
}
