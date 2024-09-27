use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use starknet_api::hash::StarkFelt;
use starknet_api::{
    block::{BlockHeader, BlockNumber, BlockStatus, BlockTimestamp},
    data_availability::L1DataAvailabilityMode,
    hash::pedersen_hash_array,
    stark_felt,
};
use starknet_devnet_types::{
    contract_address::ContractAddress,
    felt::{BlockHash, Felt, TransactionHash},
    rpc::block::{BlockHeader as TypesBlockHeader, ResourcePrice},
    traits::HashProducer,
};
use starknet_rs_core::types::BlockId;

use super::{
    constants::STARKNET_VERSION,
    errors::{DevnetResult, Error},
    starknet_state::StarknetState,
    state_diff::StateDiff,
    traits::HashIdentified,
};
#[derive(Debug)]
pub struct StarknetBlocks {
    pub num_to_hash: IndexMap<BlockNumber, BlockHash>,
    pub hash_to_block: HashMap<BlockHash, StarknetBlock>,
    pub pending_block: StarknetBlock,
    pub last_block_hash: Option<BlockHash>,
    pub hash_to_state_diff: HashMap<BlockHash, StateDiff>,
    pub hash_to_state: HashMap<BlockHash, StarknetState>,
    pub aborted_blocks: Vec<Felt>,
}

impl Serialize for StarknetBlocks {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some((highest_block_hash, highest_block)) = self
            .hash_to_block
            .iter()
            .max_by_key(|(_, block)| block.header.block_number)
        {
            let highest_state_diff = self.hash_to_state_diff.get(highest_block_hash);

            let mut state = serializer.serialize_struct("StarknetBlocks", 2)?;

            state.serialize_field("header", &highest_block.header)?;

            if let Some(state_diff) = highest_state_diff {
                state.serialize_field("state_diff", state_diff)?;
            } else {
                state.serialize_field("state_diff", &None::<StateDiff>)?;
            }

            state.end()
        } else {
            serializer.serialize_none()
        }
    }
}

impl HashIdentified for StarknetBlocks {
    type Element = StarknetBlock;
    type Hash = BlockHash;

    fn get_by_hash(&self, hash: Self::Hash) -> Option<&Self::Element> {
        let block = self.hash_to_block.get(&hash)?;

        Some(block)
    }
}

impl Default for StarknetBlocks {
    fn default() -> Self {
        Self {
            num_to_hash: IndexMap::new(),
            hash_to_block: HashMap::new(),
            pending_block: StarknetBlock::create_pending_block(),
            last_block_hash: None,
            hash_to_state_diff: HashMap::new(),
            hash_to_state: HashMap::new(),
            aborted_blocks: Vec::new(),
        }
    }
}

impl StarknetBlocks {
    pub fn new(starting_block_number: u64) -> Self {
        let mut blocks = Self::default();
        blocks.pending_block.set_block_number(starting_block_number);
        blocks
    }

    /// Inserts a block in the collection and modifies the block parent hash to match the last block
    /// hash
    pub fn insert(&mut self, mut block: StarknetBlock, state_diff: StateDiff) {
        if self.last_block_hash.is_some() {
            block.header.parent_hash = self.last_block_hash.unwrap().into();
        }

        let hash = block.block_hash();
        let block_number = block.block_number();

        self.num_to_hash.insert(block_number, hash);
        self.hash_to_block.insert(hash, block);
        self.hash_to_state_diff.insert(hash, state_diff);
        self.last_block_hash = Some(hash);
    }

    fn get_by_num(&self, num: &BlockNumber) -> Option<&StarknetBlock> {
        let block_hash = self.num_to_hash.get(num)?;
        let block = self.hash_to_block.get(block_hash)?;

        Some(block)
    }

    pub fn save_state_at(&mut self, block_hash: Felt, state: StarknetState) {
        self.hash_to_state.insert(block_hash, state);
    }

    pub fn get_by_block_id(&self, block_id: &BlockId) -> Option<&StarknetBlock> {
        match block_id {
            BlockId::Hash(hash) => self.get_by_hash(Felt::from(hash)),
            BlockId::Number(block_number) => self.get_by_num(&BlockNumber(*block_number)),
            // latest and pending for now will return the latest one
            BlockId::Tag(_) => {
                if let Some(hash) = self.last_block_hash {
                    self.get_by_hash(hash)
                } else {
                    None
                }
            }
        }
    }

    /// Returns the block number from a block id, by finding the block by the block id
    fn block_number_from_block_id(&self, block_id: &BlockId) -> Option<BlockNumber> {
        self.get_by_block_id(block_id)
            .map(|block| block.block_number())
    }

    /// Filter blocks based on from and to block ids and returns a collection of block's references
    /// in ascending order
    ///
    /// # Arguments
    /// * `from` - The block id from which to start the filtering
    /// * `to` - The block id to which to end the filtering
    pub fn get_blocks(
        &self,
        from: Option<BlockId>,
        to: Option<BlockId>,
    ) -> DevnetResult<Vec<&StarknetBlock>> {
        // used IndexMap to keep elements in the order of the keys
        let mut filtered_blocks: IndexMap<Felt, &StarknetBlock> = IndexMap::new();

        let starting_block = if let Some(block_id) = from {
            // If the value for block number provided is not correct it will return None
            // So we have to return an error
            let block_number = self
                .block_number_from_block_id(&block_id)
                .ok_or(Error::NoBlock)?;
            Some(block_number)
        } else {
            None
        };

        let ending_block = if let Some(block_id) = to {
            // if the value for block number provided is not correct it will return None
            // So we set the block number to the first possible block number which is 0
            let block_number = self
                .block_number_from_block_id(&block_id)
                .ok_or(Error::NoBlock)?;
            Some(block_number)
        } else {
            None
        };

        // iterate over the blocks and apply the filter
        // then insert the filtered blocks into the index map
        self.num_to_hash
            .iter()
            .filter(
                |(current_block_number, _)| match (starting_block, ending_block) {
                    (None, None) => true,
                    (Some(start), None) => **current_block_number >= start,
                    (None, Some(end)) => **current_block_number <= end,
                    (Some(start), Some(end)) => {
                        **current_block_number >= start && **current_block_number <= end
                    }
                },
            )
            .for_each(|(_, block_hash)| {
                filtered_blocks.insert(*block_hash, &self.hash_to_block[block_hash]);
            });

        Ok(filtered_blocks.into_values().collect())
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize)]
pub struct StarknetBlock {
    pub(crate) header: BlockHeader,
    transaction_hashes: Vec<TransactionHash>,
    pub(crate) status: BlockStatus,
}

impl From<&StarknetBlock> for TypesBlockHeader {
    fn from(value: &StarknetBlock) -> Self {
        Self {
            block_hash: value.block_hash(),
            parent_hash: value.parent_hash(),
            block_number: value.block_number(),
            sequencer_address: value.sequencer_address(),
            new_root: value.new_root(),
            timestamp: value.timestamp(),
            starknet_version: STARKNET_VERSION.to_string(),
            l1_gas_price: ResourcePrice {
                price_in_fri: value.header.l1_gas_price.price_in_fri.0.into(),
                price_in_wei: value.header.l1_gas_price.price_in_wei.0.into(),
            },
            l1_data_gas_price: ResourcePrice {
                price_in_fri: value.header.l1_data_gas_price.price_in_fri.0.into(),
                price_in_wei: value.header.l1_data_gas_price.price_in_wei.0.into(),
            },
            l1_da_mode: value.header.l1_da_mode,
        }
    }
}

impl StarknetBlock {
    pub(crate) fn add_transaction(&mut self, transaction_hash: TransactionHash) {
        self.transaction_hashes.push(transaction_hash);
    }

    pub fn get_transactions(&self) -> &Vec<TransactionHash> {
        &self.transaction_hashes
    }

    pub fn status(&self) -> &BlockStatus {
        &self.status
    }

    pub fn block_hash(&self) -> BlockHash {
        self.header.block_hash.into()
    }

    pub fn parent_hash(&self) -> BlockHash {
        self.header.parent_hash.into()
    }

    pub fn sequencer_address(&self) -> ContractAddress {
        self.header.sequencer.0.into()
    }

    pub fn timestamp(&self) -> BlockTimestamp {
        self.header.timestamp
    }

    pub fn new_root(&self) -> Felt {
        self.header.state_root.0.into()
    }

    pub(crate) fn set_block_hash(&mut self, block_hash: BlockHash) {
        self.header.block_hash = block_hash.into();
    }

    pub fn block_number(&self) -> BlockNumber {
        self.header.block_number
    }

    pub(crate) fn create_pending_block() -> Self {
        Self {
            header: BlockHeader {
                l1_da_mode: L1DataAvailabilityMode::Blob,
                ..BlockHeader::default()
            },
            status: BlockStatus::Pending,
            transaction_hashes: Vec::new(),
        }
    }

    pub(crate) fn set_block_number(&mut self, block_number: u64) {
        self.header.block_number = BlockNumber(block_number)
    }

    pub(crate) fn set_timestamp(&mut self, timestamp: BlockTimestamp) {
        self.header.timestamp = timestamp;
    }
}

impl HashProducer for StarknetBlock {
    type Error = Error;
    fn generate_hash(&self) -> DevnetResult<BlockHash> {
        let hash = pedersen_hash_array(&[
            stark_felt!(self.header.block_number.0), // block number
            self.header.state_root.0,                // global_state_root
            *self.header.sequencer.0.key(),          // sequencer_address
            stark_felt!(self.header.timestamp.0),    // block_timestamp
            stark_felt!(self.transaction_hashes.len() as u64), // transaction_count
            stark_felt!(0_u8),                       // transaction_commitment
            stark_felt!(0_u8),                       // event_count
            stark_felt!(0_u8),                       // event_commitment
            stark_felt!(0_u8),                       // protocol_version
            stark_felt!(0_u8),                       // extra_data
            stark_felt!(self.header.parent_hash.0),  // parent_block_hash
        ]);

        Ok(Felt::from(hash))
    }
}
