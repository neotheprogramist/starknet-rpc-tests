use crate::{
    assert_eq_result, assert_matches_result,
    suite_katana_no_mining::ContinuationToken,
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

use super::wait_for_sent_transaction_katana;
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, EventFilterWithPageRequest, EventsChunk};

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

        const BLOCK_1_TX_COUNT: usize = 5;
        const PENDING_BLOCK_TX_COUNT: usize = 5;

        let mut nonce = account.get_nonce().await?;

        for _ in 0..BLOCK_1_TX_COUNT {
            let res = account
                .execute_v1(vec![increase_balance_call.clone()])
                .nonce(nonce)
                .send()
                .await?;
            nonce += Felt::ONE;
            wait_for_sent_transaction_katana(res.transaction_hash, &account).await?;
        }

        // Generate new block for the pending transactions.
        dev_client.generate_block().await?;

        for _ in 0..PENDING_BLOCK_TX_COUNT {
            let res = account
                .execute_v1(vec![increase_balance_call.clone()])
                .nonce(nonce)
                .send()
                .await?;
            nonce += Felt::ONE;
            wait_for_sent_transaction_katana(res.transaction_hash, &account).await?;
        }

        let block_number = provider.block_number().await?;

        let chunk_size = BLOCK_1_TX_COUNT;

        let mut filter = EventFilterWithPageRequest {
            keys: None,
            address: None,
            to_block: None,
            from_block: Some(BlockId::Number(block_number)),
            chunk_size: chunk_size as u64,
            continuation_token: None,
        };

        let EventsChunk {
            events,
            continuation_token,
        } = provider.get_events(filter.clone()).await?;

        assert_eq_result!(events.len(), chunk_size);

        assert_matches_result!(continuation_token, Some(ref token ) => {
            let token = ContinuationToken::parse(token)?;
            assert_eq_result!(token.block_n, block_number + 1);
            assert_eq_result!(token.txn_n, 0);
            assert_eq_result!(token.event_n, 0);
        });

        // we split the pending events into two chunks to cover different cases.
        filter.continuation_token = continuation_token;
        filter.chunk_size = 3;

        let EventsChunk {
            events,
            continuation_token,
        } = provider.get_events(filter.clone()).await?;

        assert_eq_result!(events.len(), 3);

        assert_matches_result!(continuation_token, Some(ref token ) => {
            let token = ContinuationToken::parse(token)?;
            assert_eq_result!(token.block_n, block_number + 1);
            assert_eq_result!(token.txn_n, 3);
            assert_eq_result!(token.event_n, 0);
        });

        // we split the pending events into two chunks to cover different cases.
        filter.continuation_token = continuation_token;

        let EventsChunk {
            events,
            continuation_token,
        } = provider.get_events(filter.clone()).await?;

        let chunk_size = 3;

        assert_eq_result!(events.len(), PENDING_BLOCK_TX_COUNT - chunk_size as usize);
        assert_matches_result!(continuation_token, Some(ref token ) => {
            let token = ContinuationToken::parse(token)?;
            assert_eq_result!(token.block_n, block_number + 1);
            assert_eq_result!(token.txn_n, 5);
            assert_eq_result!(token.event_n, 0);
        });

        // fetching events with the continuation token should return an empty list and the
        // token shouldn't change.

        filter.continuation_token = continuation_token;

        let EventsChunk {
            events,
            continuation_token: new_token,
        } = provider.get_events(filter.clone()).await?;

        assert_eq_result!(events.len(), 0);
        assert_eq!(new_token, filter.continuation_token);

        dev_client.generate_block().await?;

        Ok(Self {})
    }
}
