use std::fmt::Display;
use std::str::FromStr;

use anyhow;
use fake::Dummy;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockHeader {
    pub hash: Felt,
    pub parent_hash: Felt,
    pub number: u64,
    pub timestamp: u64,
    pub eth_l1_gas_price: u128,
    pub strk_l1_gas_price: u128,
    pub eth_l1_data_gas_price: u128,
    pub strk_l1_data_gas_price: u128,
    pub sequencer_address: Felt,
    pub starknet_version: StarknetVersion,
    pub class_commitment: Felt,
    pub event_commitment: Felt,
    pub state_commitment: Felt,
    pub storage_commitment: Felt,
    pub transaction_commitment: Felt,
    pub transaction_count: usize,
    pub event_count: usize,
    pub l1_da_mode: L1DataAvailabilityMode,
    pub receipt_commitment: Felt,
    pub state_diff_commitment: Felt,
    pub state_diff_length: u32,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Default, Dummy, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum L1DataAvailabilityMode {
    #[default]
    Calldata,
    Blob,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SignedBlockHeader {
    pub header: BlockHeader,
    pub signature: BlockCommitmentSignature,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BlockCommitmentSignature {
    pub r: Felt,
    pub s: Felt,
}

#[derive(
    Clone, Serialize, Deserialize, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Dummy,
)]
pub struct StarknetVersion(u8, u8, u8, u8);

impl StarknetVersion {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        StarknetVersion(a, b, c, d)
    }

    pub fn as_u32(&self) -> u32 {
        u32::from_le_bytes([self.0, self.1, self.2, self.3])
    }

    pub fn from_u32(version: u32) -> Self {
        let [a, b, c, d] = version.to_le_bytes();
        StarknetVersion(a, b, c, d)
    }
}

impl FromStr for StarknetVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(StarknetVersion::new(0, 0, 0, 0));
        }

        let parts: Vec<_> = s.split('.').collect();
        anyhow::ensure!(
            parts.len() == 3 || parts.len() == 4,
            "Invalid version string, expected 3 or 4 parts but got {}",
            parts.len()
        );

        let a = parts[0].parse()?;
        let b = parts[1].parse()?;
        let c = parts[2].parse()?;
        let d = parts.get(3).map(|x| x.parse()).transpose()?.unwrap_or(0);

        Ok(StarknetVersion(a, b, c, d))
    }
}

impl Display for StarknetVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 && self.1 == 0 && self.2 == 0 && self.3 == 0 {
            return Ok(());
        }
        if self.3 == 0 {
            write!(f, "{}.{}.{}", self.0, self.1, self.2)
        } else {
            write!(f, "{}.{}.{}.{}", self.0, self.1, self.2, self.3)
        }
    }
}
