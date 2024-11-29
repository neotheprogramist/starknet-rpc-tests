use crate::{
    assert_eq_result,
    utils::v7::{
        accounts::{account::Account, call::Call},
        endpoints::{
            errors::OpenRpcTestGenError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use starknet_types_core::felt::Felt;
use starknet_types_rpc::TxnExecutionStatus;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;
        let to = Felt::from_hex_unchecked(
            "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d", //DEFAULT_STRK_FEE_TOKEN_ADDRESS
        );
        let selector = get_selector_from_name("transfer")?;
        let calldata = vec![
            Felt::from_hex_unchecked("0x1"),
            Felt::from_hex_unchecked("0x1"),
            Felt::ZERO,
        ];

        let res = account
            .execute_v3(vec![Call {
                to,
                selector,
                calldata,
            }])
            .gas(100000000000)
            .send()
            .await
            .inspect_err(|e| println!("transaction failed: {e:?}"))?;

        let status = wait_for_sent_transaction(res.transaction_hash, &account).await?;
        assert_eq_result!(status.execution_status, Some(TxnExecutionStatus::Succeeded));

        Ok(Self {})
    }
}
