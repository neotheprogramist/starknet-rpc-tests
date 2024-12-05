use crate::{
    assert_matches_result,
    utils::v7::{
        accounts::{
            account::{Account, ConnectedAccount},
            call::Call,
        },
        endpoints::{errors::OpenRpcTestGenError, utils::get_selector_from_name},
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, RunnableTrait,
};
// use starknet_core::types::TransactionTrace;
use super::wait_for_sent_transaction_katana;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::TransactionTrace;

pub const DEFAULT_PREFUNDED_ACCOUNT_BALANCE: u128 = 10 * u128::pow(10, 21);

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatanaNoMining;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;
        let provider = account.provider().clone();
        let dev_client = test_input.dev_client.clone();

        let increase_balance_call = Call {
            to: test_input.deployed_contract_address,
            selector: get_selector_from_name("increase_balance")?,
            calldata: vec![Felt::from_hex("0x50")?],
        };

        // -----------------------------------------------------------------------
        // Transactions not in pending block

        let mut hashes = Vec::new();

        let mut nonce = account.get_nonce().await?;

        for _ in 0..2 {
            let res = account
                .execute_v1(vec![increase_balance_call.clone()])
                .nonce(nonce)
                .send()
                .await?;
            wait_for_sent_transaction_katana(res.transaction_hash, &account).await?;
            nonce += Felt::ONE;
            hashes.push(res.transaction_hash);
        }

        // Generate a block to include the transactions.
        dev_client.generate_block().await?;

        for hash in hashes {
            let trace = provider.trace_transaction(hash).await?;
            assert_matches_result!(trace, TransactionTrace::Invoke(_));
        }

        // -----------------------------------------------------------------------
        // Transactions in pending block
        for _ in 0..2 {
            let res = account
                .execute_v1(vec![increase_balance_call.clone()])
                .nonce(nonce)
                .send()
                .await?;
            wait_for_sent_transaction_katana(res.transaction_hash, &account).await?;
            nonce += Felt::ONE;
            let trace = provider.trace_transaction(res.transaction_hash).await?;
            assert_matches_result!(trace, TransactionTrace::Invoke(_));
        }

        // Generate new block for the pending transactions.
        dev_client.generate_block().await?;

        Ok(Self {})
    }
}
