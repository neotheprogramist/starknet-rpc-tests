use starknet_types_core::felt::Felt;
use starknet_types_core::hash::{Pedersen, StarkHash};
#[derive(Default)]
pub struct HashChain {
    hash: Felt,
    count: usize,
}

impl HashChain {
    pub fn update(&mut self, value: Felt) {
        self.hash = Pedersen::hash(&self.hash, &value);
        self.count = self
            .count
            .checked_add(1)
            .expect("could not have deserialized larger than usize Vecs");
    }

    pub fn chain_update(mut self, value: Felt) -> Self {
        self.update(value);
        self
    }

    pub fn finalize(self) -> Felt {
        let count = Felt::from_bytes_be_slice(&self.count.to_be_bytes());
        Pedersen::hash(&self.hash, &count)
    }

    pub fn single(value: Felt) -> Felt {
        Self::default().chain_update(value).finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::{Felt, HashChain};

    #[test]
    fn test_non_empty_chain() {
        let mut chain = HashChain::default();

        chain.update(Felt::from_hex_unchecked("0x1"));
        chain.update(Felt::from_hex_unchecked("0x2"));
        chain.update(Felt::from_hex_unchecked("0x3"));
        chain.update(Felt::from_hex_unchecked("0x4"));

        let computed_hash = chain.finalize();

        // produced by the cairo-lang Python implementation:
        // `hex(compute_hash_on_elements([1, 2, 3, 4]))`
        let expected_hash = Felt::from_hex_unchecked(
            "0x66bd4335902683054d08a0572747ea78ebd9e531536fb43125424ca9f902084",
        );

        assert_eq!(expected_hash, computed_hash);
    }
}
