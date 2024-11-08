use bitvec::prelude::Msb0;
use bitvec::vec::BitVec;
use starknet_types_core::felt::Felt;

use super::hash::FeltHash;

/// A node in a Starknet patricia-merkle trie.
///
/// See pathfinders merkle-tree crate for more information.
#[derive(Debug, Clone, PartialEq)]
pub enum TrieNode {
    Binary { left: Felt, right: Felt },
    Edge { child: Felt, path: BitVec<u8, Msb0> },
}

impl TrieNode {
    pub fn hash<H: FeltHash>(&self) -> Felt {
        match self {
            TrieNode::Binary { left, right } => H::hash(*left, *right),
            TrieNode::Edge { child, path } => {
                let mut length = [0; 32];
                length[31] = path.len() as u8;

                // Convert the `BitSlice<u8, Msb0>` to a `Vec<u8>`
                let mut path_bytes = vec![0u8; (path.len() + 7) / 8];
                path.as_bitslice().iter().enumerate().for_each(|(i, bit)| {
                    if *bit {
                        path_bytes[i / 8] |= 1 << (7 - (i % 8));
                    }
                });

                let path_felt = Felt::from_bytes_be_slice(&path_bytes);

                let length_felt = Felt::from_bytes_be(&length);
                H::hash(*child, path_felt) + length_felt
            }
        }
    }
}
