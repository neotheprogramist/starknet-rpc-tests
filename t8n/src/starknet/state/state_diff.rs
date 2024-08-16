use std::collections::HashMap;

use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::{State, StateReader};
use serde::Serialize;
use starknet_devnet_types::contract_address::ContractAddress;
use starknet_devnet_types::error::DevnetResult;
use starknet_devnet_types::felt::{ClassHash, Felt};
use starknet_devnet_types::patricia_key::{PatriciaKey, StorageKey};
use starknet_devnet_types::rpc::state::{
    ClassHashes, ContractNonce, DeployedContract, StorageDiff, StorageEntry, ThinStateDiff,
};

use super::starknet_state::CommittedClassStorage;

/// This struct is used to store the difference between state modifications
#[derive(PartialEq, Default, Debug, Clone, Serialize)]
pub struct StateDiff {
    pub(crate) storage_updates: HashMap<ContractAddress, HashMap<StorageKey, Felt>>,
    pub(crate) address_to_nonce: HashMap<ContractAddress, Felt>,
    pub(crate) address_to_class_hash: HashMap<ContractAddress, ClassHash>,
    // class hash to compiled_class_hash difference, used when declaring contracts
    // that are different from cairo 0
    pub(crate) class_hash_to_compiled_class_hash: HashMap<ClassHash, ClassHash>,
    // declare contracts that are not cairo 0
    pub(crate) declared_contracts: Vec<ClassHash>,
    // cairo 0 declared contracts
    pub(crate) cairo_0_declared_contracts: Vec<ClassHash>,
}

impl Eq for StateDiff {}

impl StateDiff {
    pub(crate) fn generate<S: StateReader>(
        state: &mut CachedState<S>,
        contract_classes: &mut CommittedClassStorage,
    ) -> DevnetResult<Self> {
        let mut declared_contracts = Vec::<ClassHash>::new();
        let mut cairo_0_declared_contracts = Vec::<ClassHash>::new();

        let diff = state.to_state_diff();
        state.move_classes_to_global_cache();
        let new_classes = contract_classes.commit();

        for (class_hash, class) in new_classes {
            match class {
                starknet_devnet_types::contract_class::ContractClass::Cairo0(_) => {
                    cairo_0_declared_contracts.push(class_hash)
                }
                starknet_devnet_types::contract_class::ContractClass::Cairo1(_) => {
                    declared_contracts.push(class_hash)
                }
            }
        }

        // extract differences of class_hash -> compile_class_hash mapping
        let class_hash_to_compiled_class_hash = diff
            .class_hash_to_compiled_class_hash
            .into_iter()
            .map(|(class_hash, compiled_class_hash)| {
                (Felt::from(class_hash.0), Felt::from(compiled_class_hash.0))
            })
            .collect();

        let address_to_class_hash = diff
            .address_to_class_hash
            .iter()
            .map(|(address, class_hash)| {
                let contract_address = ContractAddress::from(*address);
                let class_hash = class_hash.0.into();

                (contract_address, class_hash)
            })
            .collect::<HashMap<ContractAddress, ClassHash>>();

        let address_to_nonce = diff
            .address_to_nonce
            .iter()
            .map(|(address, nonce)| {
                let contract_address = ContractAddress::from(*address);
                let nonce = nonce.0.into();

                (contract_address, nonce)
            })
            .collect::<HashMap<ContractAddress, Felt>>();

        let storage_updates = diff
            .storage_updates
            .iter()
            .map(|(address, storage)| {
                let contract_address = ContractAddress::from(*address);
                let storage = storage
                    .iter()
                    .map(|(key, value)| {
                        let key = PatriciaKey::from(key.0);
                        let value = (*value).into();

                        (key, value)
                    })
                    .collect::<HashMap<StorageKey, Felt>>();

                (contract_address, storage)
            })
            .collect::<HashMap<ContractAddress, HashMap<StorageKey, Felt>>>();

        Ok(StateDiff {
            address_to_class_hash,
            address_to_nonce,
            storage_updates,
            class_hash_to_compiled_class_hash,
            cairo_0_declared_contracts,
            declared_contracts,
        })
    }
}

impl From<StateDiff> for ThinStateDiff {
    fn from(value: StateDiff) -> Self {
        let declared_classes: Vec<(Felt, Felt)> = value
            .class_hash_to_compiled_class_hash
            .into_iter()
            .collect();

        // cairo 0 declarations
        let cairo_0_declared_classes: Vec<Felt> = value.cairo_0_declared_contracts;

        // storage updates (contract address -> [(storage_entry, value)])
        let storage_updates: Vec<(ContractAddress, Vec<(PatriciaKey, Felt)>)> = value
            .storage_updates
            .into_iter()
            .map(|(address, entries)| (address, entries.into_iter().collect()))
            .collect();

        // contract nonces
        let nonces: Vec<(ContractAddress, Felt)> = value.address_to_nonce.into_iter().collect();

        // deployed contracts (address -> class hash)
        let deployed_contracts: Vec<(ContractAddress, Felt)> =
            value.address_to_class_hash.into_iter().collect();

        ThinStateDiff {
            deployed_contracts: deployed_contracts
                .into_iter()
                .map(|(address, class_hash)| DeployedContract {
                    address,
                    class_hash,
                })
                .collect(),
            declared_classes: declared_classes
                .into_iter()
                .map(|(class_hash, compiled_class_hash)| ClassHashes {
                    class_hash,
                    compiled_class_hash,
                })
                .collect(),
            deprecated_declared_classes: cairo_0_declared_classes,
            nonces: nonces
                .into_iter()
                .map(|(address, nonce)| ContractNonce {
                    contract_address: address,
                    nonce,
                })
                .collect(),
            storage_diffs: storage_updates
                .into_iter()
                .map(|(contract_address, updates)| StorageDiff {
                    address: contract_address,
                    storage_entries: updates
                        .into_iter()
                        .map(|(key, value)| StorageEntry { key, value })
                        .collect(),
                })
                .collect(),
            replaced_classes: vec![],
        }
    }
}
