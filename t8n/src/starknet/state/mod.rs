pub mod constants;
pub mod defaulter;
pub mod dict_state;
pub mod errors;
pub mod starknet_state;
pub mod state_diff;
pub mod types;
pub mod utils;
use blockifier::context::BlockContext;
use starknet_state::StarknetState;

pub struct Starknet {
    pub state: StarknetState,
    pub block_context: BlockContext,
}
