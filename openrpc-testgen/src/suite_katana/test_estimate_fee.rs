use crate::{
    assert_result,
    utils::v7::{
        accounts::{
            account::{Account, ConnectedAccount},
            call::Call,
        },
        endpoints::{
            errors::OpenRpcTestGenError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;
        let provider = account.provider().clone();

        // prepare Call
        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        // send a valid transaction first to increment the nonce (so that we can test nonce < current
        // nonce later)
        let res = account
            .execute_v1(vec![increase_balance_call.clone()])
            .send()
            .await?;

        wait_for_sent_transaction(res.transaction_hash, &account).await?;

        // estimate fee with current nonce (the expected nonce)
        let nonce = provider
            .get_nonce(BlockId::Tag(BlockTag::Pending), account.address())
            .await?;
        let result = account
            .execute_v1(vec![increase_balance_call.clone()])
            .nonce(nonce)
            .estimate_fee()
            .await;
        let result_is_ok: bool = result.is_ok();

        assert_result!(
            result_is_ok,
            "estimate should succeed with nonce == current nonce"
        );

        // estimate fee with arbitrary nonce < current nonce
        //
        // here we're essentially estimating a transaction with a nonce that has already been
        // used, so it should fail.
        let nonce = nonce - 1;
        let result = account
            .execute_v1(vec![increase_balance_call.clone()])
            .nonce(nonce)
            .estimate_fee()
            .await;
        let result_is_err = result.is_err();

        assert_result!(
            result_is_err,
            "estimate should fail with nonce < current nonce"
        );

        // estimate fee with arbitrary nonce >= current nonce
        let nonce = Felt::from_hex_unchecked("0x1337");
        let result = account
            .execute_v1(vec![increase_balance_call.clone()])
            .nonce(nonce)
            .estimate_fee()
            .await;
        let result_is_ok: bool = result.is_ok();

        assert_result!(
            result_is_ok,
            "estimate should succeed with nonce >= current nonce"
        );

        Ok(Self {})
    }
}
