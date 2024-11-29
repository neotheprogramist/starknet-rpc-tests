use crate::{
    assert_matches_result,
    utils::v7::{
        accounts::{
            account::{Account, AccountError, ConnectedAccount},
            call::Call,
            creation::create::{create_account, AccountType},
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

        let account_data = create_account(
            test_input.random_paymaster_account.provider(),
            AccountType::Oz,
            Option::None,
            Some(test_input.account_class_hash),
        )
        .await?;

        let transfer_amount = Felt::from(DEFAULT_PREFUNDED_ACCOUNT_BALANCE / 2);

        let transfer_execution = account
            .execute_v1(vec![Call {
                to: Felt::from_hex(
                    "0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7",
                )?,
                selector: get_selector_from_name("transfer")?,
                calldata: vec![account_data.address, transfer_amount, Felt::ZERO],
            }])
            .send()
            .await?;

        wait_for_sent_transaction(transfer_execution.transaction_hash, &account).await?;

        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        let invoke_result = test_input
            .random_paymaster_account
            .execute_v1(vec![increase_balance_call])
            .max_fee(DEFAULT_PREFUNDED_ACCOUNT_BALANCE.into())
            .send()
            .await;

        assert_matches_result!(
            invoke_result.unwrap_err(),
            AccountError::Provider(ProviderError::StarknetError(
                StarknetError::InsufficientAccountBalance
            ))
        );

        Ok(Self {})
    }
}

pub fn split_felt(felt: Felt) -> (Felt, Felt) {
    let low: Felt = (felt.to_biguint() & Felt::from(u128::MAX).to_biguint()).into();
    let high = felt.to_biguint() >> 128;
    (low, Felt::from(high))
}
