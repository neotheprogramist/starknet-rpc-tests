//! Contains the [FeltHash] trait and implementations thereof for the
//! [Pedersen](PedersenHash) and [Poseidon](PoseidonHash) hashes.
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::{Pedersen, Poseidon, StarkHash};

/// Allows for implementations to be generic over Felt hash functions.
///
/// Implemented by [PedersenHash] and [PoseidonHash].
pub trait FeltHash {
    fn hash(a: Felt, b: Felt) -> Felt;
}

/// Implements [Hash] for the [Starknet Pedersen hash](pedersen_hash).
#[derive(Debug, Clone, Copy)]
pub struct PedersenHash {}

impl FeltHash for PedersenHash {
    fn hash(a: Felt, b: Felt) -> Felt {
        Pedersen::hash(&a, &b)
    }
}

/// Implements [Hash] for the [Starknet Poseidon hash](poseidon_hash).
#[derive(Debug, Clone, Copy)]
pub struct PoseidonHash;
impl FeltHash for PoseidonHash {
    fn hash(a: Felt, b: Felt) -> Felt {
        Poseidon::hash(&a, &b)
    }
}
