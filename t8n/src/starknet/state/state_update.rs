use starknet_rs_core::types::BlockId;

use super::{
    errors::{DevnetResult, Error},
    Starknet,
};

use starknet_devnet_types::felt::Felt;

use super::state_diff::StateDiff;

#[derive(Debug)]
pub struct StateUpdate {
    pub block_hash: Felt,
    pub new_root: Felt,
    pub old_root: Felt,
    pub state_diff: StateDiff,
}

impl StateUpdate {
    pub fn new(block_hash: Felt, state_diff: StateDiff) -> Self {
        // TODO new and old root are not computed, they are not part of the MVP
        Self {
            block_hash,
            new_root: Felt::default(),
            old_root: Felt::default(),
            state_diff,
        }
    }
}

pub fn state_update_by_block_id(
    starknet: &Starknet,
    block_id: &BlockId,
) -> DevnetResult<StateUpdate> {
    let block = starknet
        .blocks
        .get_by_block_id(block_id)
        .ok_or(Error::NoBlock)?;
    let state_diff = starknet
        .blocks
        .hash_to_state_diff
        .get(&block.block_hash())
        .cloned()
        .unwrap_or_default();

    Ok(StateUpdate::new(block.block_hash(), state_diff))
}
