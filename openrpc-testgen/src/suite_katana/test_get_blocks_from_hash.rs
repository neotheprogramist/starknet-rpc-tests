use crate::{
    assert_matches_result, assert_result,
    utils::v7::{
        accounts::account::{Account, AccountError, ConnectedAccount},
        endpoints::{
            declare_contract::get_compiled_contract, errors::RpcError,
            utils::wait_for_sent_transaction,
        },
        providers::{
            jsonrpc::StarknetError,
            provider::{Provider, ProviderError},
        },
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag, MaybePendingBlockWithTxHashes, MaybePendingBlockWithTxs};

use std::{path::PathBuf, str::FromStr, sync::Arc};

#[derive(Clone, Debug)]
pub struct TestCase {
    pub result: Result<(), String>,
}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let provider = test_input
            .random_paymaster_account
            .random_accounts()?
            .provider()
            .clone();

        // -----------------------------------------------------------------------
        // Get the forked block

        // https://sepolia.voyager.online/block/0x208950cfcbba73ecbda1c14e4d58d66a8d60655ea1b9dcf07c16014ae8a93cd
        let hash = Felt::from_hex_unchecked(
            "0x208950cfcbba73ecbda1c14e4d58d66a8d60655ea1b9dcf07c16014ae8a93cd",
        ); // 268471
        let id = BlockId::Hash(hash);

        let block = provider.get_block_with_txs(id).await?;
        assert_matches_result!(block, MaybePendingBlockWithTxs::Block(b) if b.block_header.block_hash == hash);
        // TODO: impolement get_block_with_receipts method
        // let block = provider.get_block_with_receipts(id).await?;
        // assert_matches_result!(block, StarknetGetBlockWithTxsAndReceiptsResult::Block(b) if b.block_hash == hash);

        let block = provider.get_block_with_tx_hashes(id).await?;
        assert_matches_result!(block, MaybePendingBlockWithTxHashes::Block(b) if b.block_header.block_hash == hash);

        let result = provider.get_block_transaction_count(id).await;
        assert!(result.is_ok());

        // TODO: uncomment this once we include genesis forked state update
        // let state = provider.get_state_update(id).await?;
        // assert_matches!(state, starknet::core::types::MaybePendingStateUpdate::Update(_));

        // -----------------------------------------------------------------------
        // Get a block before the forked block
        // https://sepolia.voyager.online/block/0x42dc67af5003d212ac6cd784e72db945ea4d619898f30f422358ff215cbe1e4

        let hash = Felt::from_hex_unchecked("0x42dc67af5003d212ac6cd784e72db945ea4d619898f30f422358ff215cbe1e4"); // 268466
        let id = BlockId::Hash(hash);

        let block = provider.get_block_with_txs(id).await?;
        assert_matches_result!(block, MaybePendingBlockWithTxs::Block(b) if b.block_header.block_hash == hash);
        // TODO: impolement get_block_with_receipts method
        // let block = provider.get_block_with_receipts(id).await?;
        // assert_matches_result!(block, MaybePendingBlockWithReceipts::Block(b) if b.block_hash == hash);

        let block = provider.get_block_with_tx_hashes(id).await?;
        assert_matches_result!(block, MaybePendingBlockWithTxHashes::Block(b) if b.block_header.block_hash == hash);

        let result = provider.get_block_transaction_count(id).await;
        assert_result!(result.is_ok());

        // TODO: uncomment this once we include genesis forked state update
        // let state = provider.get_state_update(id).await?;
        // assert_matches!(state, starknet::core::types::MaybePendingStateUpdate::Update(_));

        // -----------------------------------------------------------------------
        // Get a block that is locally generated
        // TODO: Get local blocks from the provider
        // for ((_, hash), _) in local_only_block {
        //     let id = BlockId::Hash(hash);

        //     let block = provider.get_block_with_txs(id).await?;
        //     assert_matches_result!(block, MaybePendingBlockWithTxs::Block(b) if b.block_hash == hash);

        //     let block = provider.get_block_with_receipts(id).await?;
        //     assert_matches_result!(block, MaybePendingBlockWithReceipts::Block(b) if b.block_hash == hash);

        //     let block = provider.get_block_with_tx_hashes(id).await?;
        //     assert_matches_result!(block, MaybePendingBlockWithTxHashes::Block(b) if b.block_hash == hash);

        //     let result = provider.get_block_transaction_count(id).await;
        //     assert_result!(result.is_ok());

            // TODO: uncomment this once we include genesis forked state update
            // let state = provider.get_state_update(id).await?;
            // assert_matches!(state, starknet::core::types::MaybePendingStateUpdate::Update(_));
        // }

        // -----------------------------------------------------------------------
        // Get a block that only exist in the forked chain

        // https://sepolia.voyager.online/block/0x347a9fa25700e7a2d8f26b39c0ecf765be9a78c559b9cae722a659f25182d10
        // We only created 10 local blocks so this is fine.
        let id = BlockId::Number(270_328);
        let result = provider.get_block_with_txs(id).await.unwrap_err();
        assert_matches_result!(result, ProviderError::StarknetError(StarknetError::BlockNotFound));

        // TODO: impolement get_block_with_receipts method        
        // let result = provider.get_block_with_receipts(id).await.unwrap_err();
        // assert_matches_result!(result, StarknetError::BlockNotFound);

        let result = provider.get_block_with_tx_hashes(id).await.unwrap_err();
        assert_matches_result!(result, ProviderError::StarknetError(StarknetError::BlockNotFound));

        let result = provider.get_block_transaction_count(id).await.unwrap_err();
        assert_matches_result!(result,  ProviderError::StarknetError(StarknetError::BlockNotFound));

        let result = provider.get_state_update(id).await.unwrap_err();
        assert_matches_result!(result, ProviderError::StarknetError(StarknetError::BlockNotFound));

        // -----------------------------------------------------------------------
        // Get block that doesn't exist on the both the forked and local chain

        let id = BlockId::Number(i64::MAX as u64);
        let result = provider.get_block_with_txs(id).await.unwrap_err();
        assert_matches_result!(result,  ProviderError::StarknetError(StarknetError::BlockNotFound));
        
        // TODO: impolement get_block_with_receipts method        
        // let result = provider.get_block_with_receipts(id).await.unwrap_err();
        // assert_provider_starknet_err!(result,  ProviderError::StarknetError(StarknetError::BlockNotFound));

        let result = provider.get_block_with_tx_hashes(id).await.unwrap_err();
        assert_matches_result!(result,  ProviderError::StarknetError(StarknetError::BlockNotFound));

        let result = provider.get_block_transaction_count(id).await.unwrap_err();
        assert_matches_result!(result,  ProviderError::StarknetError(StarknetError::BlockNotFound));

        let result = provider.get_state_update(id).await.unwrap_err();
        assert_matches_result!(result, ProviderError::StarknetError(StarknetError::BlockNotFound));

        Ok(Self { result: Ok(()) })
    }
}
