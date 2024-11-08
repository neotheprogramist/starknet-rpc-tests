use std::collections::{HashMap, HashSet};

use crate::pathfinder_types::types::reply;
use crate::pathfinder_types::types::reply::state_update::StorageDiff;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use starknet_types_core::felt::Felt;

/// This struct is used to store the difference between state modifications
#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub storage_updates: HashMap<Felt, HashMap<Felt, Felt>>,
    pub address_to_nonce: HashMap<Felt, Felt>,
    pub address_to_class_hash: HashMap<Felt, Felt>,
    // class hash to compiled_class_hash difference, used when declaring contracts
    // that are different from cairo 0
    pub class_hash_to_compiled_class_hash: HashMap<Felt, Felt>,
    // declare contracts that are not cairo 0
    pub declared_contracts: Vec<Felt>,
    // cairo 0 declared contracts
    pub cairo_0_declared_contracts: Vec<Felt>,
}

#[derive(PartialEq, Default, Debug, Clone, Serialize)]
pub struct HashToStateDiff {
    pub block_hash: Felt,
    pub state_diff: StateDiff,
}

impl<'de> Deserialize<'de> for HashToStateDiff {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<Felt, StateDiff> = HashMap::deserialize(deserializer)?;
        if map.len() != 1 {
            return Err(D::Error::custom(
                "Expected exactly one block hash in hash_to_state_diff",
            ));
        }

        let (block_hash, state_diff) = map.into_iter().next().unwrap();
        Ok(HashToStateDiff {
            block_hash,
            state_diff,
        })
    }
}

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize)]
pub struct BlockStateDiff {
    #[serde(deserialize_with = "deserialize_hash_to_state_diff")]
    pub hash_to_state_diff: HashToStateDiff,
}

fn deserialize_hash_to_state_diff<'de, D>(deserializer: D) -> Result<HashToStateDiff, D::Error>
where
    D: Deserializer<'de>,
{
    HashToStateDiff::deserialize(deserializer)
}

impl Eq for StateDiff {}

impl From<StateDiff> for reply::state_update::StateDiff {
    fn from(value: StateDiff) -> Self {
        let declared_classes = value
            .class_hash_to_compiled_class_hash
            .into_iter()
            .map(
                |(class_hash, compiled_class_hash)| reply::state_update::DeclaredSierraClass {
                    class_hash,
                    compiled_class_hash,
                },
            )
            .collect();

        let deployed_contracts = value
            .address_to_class_hash
            .into_iter()
            .map(
                |(address, class_hash)| reply::state_update::DeployedContract {
                    address,
                    class_hash,
                },
            )
            .collect();

        let old_declared_contracts: HashSet<Felt> =
            value.cairo_0_declared_contracts.into_iter().collect();

        let storage_updates: HashMap<Felt, Vec<(Felt, Felt)>> = value
            .storage_updates
            .into_iter()
            .map(|(address, entries)| (address, entries.into_iter().collect()))
            .collect();

        let storage_diffs = storage_updates
            .into_iter()
            .map(|(contract_address, updates)| {
                let storage_entries = updates
                    .into_iter()
                    .map(|(key, value)| StorageDiff { key, value })
                    .collect::<Vec<StorageDiff>>();

                (contract_address, storage_entries)
            })
            .collect::<HashMap<Felt, Vec<StorageDiff>>>();

        let nonces: HashMap<
            reply::state_update::ContractAddress,
            reply::state_update::ContractNonce,
        > = value.address_to_nonce.into_iter().collect();

        let replaced_classes = Vec::new();

        reply::state_update::StateDiff {
            deployed_contracts,
            storage_diffs,
            declared_classes,
            old_declared_contracts,
            nonces,
            replaced_classes,
        }
    }
}
