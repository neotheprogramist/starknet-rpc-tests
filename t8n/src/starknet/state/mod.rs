pub mod constants;
pub mod defaulter;
pub mod dict_state;
pub mod errors;
pub mod starknet_blocks;
pub mod starknet_state;
pub mod starknet_transactions;
pub mod state_diff;
pub mod traits;
pub mod types;
pub mod utils;
use blockifier::context::BlockContext;
use starknet_blocks::StarknetBlocks;
use starknet_state::StarknetState;
use starknet_transactions::StarknetTransactions;

pub struct Starknet {
    pub state: StarknetState,
    pub block_context: BlockContext,
    // To avoid repeating some logic related to blocks,
    // having `blocks` public allows to re-use functions like `get_blocks()`.
    pub blocks: StarknetBlocks,
    pub transactions: StarknetTransactions,
}
