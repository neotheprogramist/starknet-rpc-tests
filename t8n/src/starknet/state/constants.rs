// Casm hash calculation-related constants.
pub const CAIRO0_ENTRY_POINT_STRUCT_SIZE: usize = 2;
pub const N_STEPS_PER_PEDERSEN: usize = 8;

// OS reserved contract addresses.

// This contract stores the block number -> block hash mapping.
// TODO(Arni, 14/6/2023): Replace BLOCK_HASH_CONSTANT_ADDRESS with a lazy calculation.
//      pub static BLOCK_HASH_CONTRACT_ADDRESS: Lazy<ContractAddress> = ...
pub const BLOCK_HASH_CONTRACT_ADDRESS: u64 = 1;

// The block number -> block hash mapping is written for the current block number minus this number.
pub const STORED_BLOCK_HASH_BUFFER: u64 = 10;

pub const STARKNET_VERSION: &str = "0.13.1.1";

pub const UDC_CONTRACT_ADDRESS: &str =
    "0x41A78E741E5AF2FEC34B695679BC6891742439F7AFB8484ECD7766661AD02BF";
