//! Contains constructs for describing the nodes in a Binary Merkle Patricia
//! Tree used by Starknet.
//!
//! For more information about how these Starknet trees are structured, see
//! [`MerkleTree`](crate::tree::MerkleTree).

use std::cell::RefCell;
use std::rc::Rc;

use bitvec::order::Msb0;
use bitvec::prelude::*;
use bitvec::slice::BitSlice;

use super::hash::FeltHash;
use starknet_types_core::felt::Felt;

/// A node in a Binary Merkle-Patricia Tree graph.
#[derive(Clone, Debug, PartialEq)]
pub enum InternalNode {
    /// A node that has not been fetched from storage yet.
    ///
    /// As such, all we know is its index.
    Unresolved(u64),
    /// A branch node with exactly two children.
    Binary(BinaryNode),
    /// Describes a path connecting two other nodes.
    Edge(EdgeNode),
    /// A leaf node.
    Leaf,
}

/// Describes the [InternalNode::Binary] variant.
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryNode {
    /// The storage index of this node (if it was loaded from storage).
    pub storage_index: Option<u64>,
    /// The height of this node in the tree.
    pub height: usize,
    /// [Left](Direction::Left) child.
    pub left: Rc<RefCell<InternalNode>>,
    /// [Right](Direction::Right) child.
    pub right: Rc<RefCell<InternalNode>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EdgeNode {
    /// The storage index of this node (if it was loaded from storage).
    pub storage_index: Option<u64>,
    /// The starting height of this node in the tree.
    pub height: usize,
    /// The path this edge takes.
    pub path: BitVec<u8, Msb0>,
    /// The child of this node.
    pub child: Rc<RefCell<InternalNode>>,
}

/// Describes the direction a child of a [BinaryNode] may have.
///
/// Binary nodes have two children, one left and one right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    /// Inverts the [Direction].
    ///
    /// [Left] becomes [Right], and [Right] becomes [Left].
    ///
    /// [Left]: Direction::Left
    /// [Right]: Direction::Right
    pub fn invert(self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl From<bool> for Direction {
    fn from(tf: bool) -> Self {
        match tf {
            true => Direction::Right,
            false => Direction::Left,
        }
    }
}

impl From<Direction> for bool {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Left => false,
            Direction::Right => true,
        }
    }
}

impl BinaryNode {
    /// Maps the key's bit at the binary node's height to a [Direction].
    ///
    /// This can be used to check which direction the key describes in the
    /// context of this binary node i.e. which direction the child along the
    /// key's path would take.
    pub fn direction(&self, key: &BitSlice<u8, Msb0>) -> Direction {
        key[self.height].into()
    }

    /// Returns the [Left] or [Right] child.
    ///
    /// [Left]: Direction::Left
    /// [Right]: Direction::Right
    pub fn get_child(&self, direction: Direction) -> Rc<RefCell<InternalNode>> {
        match direction {
            Direction::Left => self.left.clone(),
            Direction::Right => self.right.clone(),
        }
    }

    pub(crate) fn calculate_hash<H: FeltHash>(left: Felt, right: Felt) -> Felt {
        H::hash(left, right)
    }
}

impl InternalNode {
    pub fn is_binary(&self) -> bool {
        matches!(self, InternalNode::Binary(..))
    }

    pub fn as_binary(&self) -> Option<&BinaryNode> {
        match self {
            InternalNode::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    pub fn as_edge(&self) -> Option<&EdgeNode> {
        match self {
            InternalNode::Edge(edge) => Some(edge),
            _ => None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, InternalNode::Leaf)
    }

    pub fn storage_index(&self) -> Option<u64> {
        match self {
            InternalNode::Unresolved(storage_index) => Some(*storage_index),
            InternalNode::Binary(binary) => binary.storage_index,
            InternalNode::Edge(edge) => edge.storage_index,
            InternalNode::Leaf => None,
        }
    }
}

impl EdgeNode {
    /// Returns true if the edge node's path matches the same path given by the
    /// key.
    pub fn path_matches(&self, key: &BitSlice<u8, Msb0>) -> bool {
        self.path == key[self.height..self.height + self.path.len()]
    }

    /// Returns the common bit prefix between the edge node's path and the given
    /// key.
    ///
    /// This is calculated with the edge's height taken into account.
    pub fn common_path(&self, key: &BitSlice<u8, Msb0>) -> &BitSlice<u8, Msb0> {
        let key_path = key.iter().skip(self.height);
        let common_length = key_path
            .zip(self.path.iter())
            .take_while(|(a, b)| a == b)
            .count();

        &self.path[..common_length]
    }

    pub(crate) fn calculate_hash<H: FeltHash>(child: Felt, path: &BitSlice<u8, Msb0>) -> Felt {
        let mut length = [0; 32];
        // Safe as len() is guaranteed to be <= 251
        length[31] = path.len() as u8;
        let length = Felt::from_bytes_be(&length);

        let mut bytes = [0u8; 32];
        bytes.view_bits_mut::<Msb0>()[256 - path.len()..].copy_from_bitslice(path);

        // Create `Felt` from the byte slice in big-endian format
        let path_felt = Felt::from_bytes_be(&bytes);

        H::hash(child, path_felt) + length
    }
}

#[cfg(test)]
mod tests {
    use crate::pathfinder_types::types::hash::PedersenHash;
    use crate::pathfinder_types::types::merkle_node::EdgeNode;
    use bitvec::order::Msb0;
    use bitvec::prelude::*;

    use starknet_types_core::felt::Felt;

    #[test]
    fn hash() {
        // Test data taken from starkware cairo-lang repo:
        // https://github.com/starkware-libs/cairo-lang/blob/fc97bdd8322a7df043c87c371634b26c15ed6cee/src/starkware/starkware_utils/commitment_tree/patricia_tree/nodes_test.py#L38
        //
        // Note that the hash function must be exchanged for `async_stark_hash_func`,
        // otherwise it just uses some other test hash function.
        let expected = Felt::from_hex_unchecked(
            "0x1d937094c09b5f8e26a662d21911871e3cbc6858d55cc49af9848ea6fed4e9",
        );
        // .unwrap();
        let child = Felt::from_hex_unchecked("0x1234ABCD");

        // Path = 42 in binary.
        let path = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0];

        let hash = EdgeNode::calculate_hash::<PedersenHash>(child, &path);
        assert_eq!(hash, expected);
    }
}
