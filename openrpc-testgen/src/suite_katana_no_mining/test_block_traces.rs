use crate::{
    assert_eq_result, assert_matches_result,
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
use starknet_types_rpc::{BlockId, BlockTag, TransactionTrace};

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

        let mut hashes = Vec::new();

        let mut nonce = account.get_nonce().await?;

        // -----------------------------------------------------------------------
        // Block 1

        for _ in 0..5 {
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

        // Get the traces of the transactions in block.
        let block_id = BlockId::Tag(BlockTag::Latest);
        let traces = provider.trace_block_transactions(block_id).await?;
        assert_eq_result!(traces.len(), 5);

        for i in 0..5 {
            assert_eq_result!(traces[i].transaction_hash, Some(hashes[i]));
            assert_matches_result!(&traces[i].trace_root, Some(TransactionTrace::Invoke(_)));
        }

        // -----------------------------------------------------------------------
        // Block 2

        // remove the previous transaction hashes
        hashes.clear();

        nonce = account.get_nonce().await?;

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

        // Generate a block to include the transactions. The generated block will have block number 2.
        dev_client.generate_block().await?;

        // Get the traces of the transactions in block.
        let block_id = BlockId::Tag(BlockTag::Latest);
        let traces = provider.trace_block_transactions(block_id).await?;
        assert_eq_result!(traces.len(), 2);

        for i in 0..2 {
            assert_eq_result!(traces[i].transaction_hash, Some(hashes[i]));
            assert_matches_result!(&traces[i].trace_root, Some(TransactionTrace::Invoke(_)));
        }

        // -----------------------------------------------------------------------
        // Block 3 (Pending)

        // remove the previous transaction hashes
        hashes.clear();

        nonce = account.get_nonce().await?;

        for _ in 0..3 {
            let res = account
                .execute_v1(vec![increase_balance_call.clone()])
                .nonce(nonce)
                .send()
                .await?;
            wait_for_sent_transaction_katana(res.transaction_hash, &account).await?;
            nonce += Felt::ONE;
            hashes.push(res.transaction_hash);
        }

        // Get the traces of the transactions in block 3 (pending).
        let block_id = BlockId::Tag(BlockTag::Pending);
        let traces = provider.trace_block_transactions(block_id).await?;
        assert_eq!(traces.len(), 3);

        for i in 0..3 {
            assert_eq_result!(traces[i].transaction_hash, Some(hashes[i]));
            assert_matches_result!(&traces[i].trace_root, Some(TransactionTrace::Invoke(_)));
        }

        // Generate new block for the pending transactions.
        dev_client.generate_block().await?;

        Ok(Self {})
    }
}
