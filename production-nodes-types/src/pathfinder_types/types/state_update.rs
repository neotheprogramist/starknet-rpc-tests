use starknet_types_core::felt::Felt;
use std::collections::{HashMap, HashSet};

pub type BlockHash = Felt;
pub type CasmHash = Felt;
pub type ClassHash = Felt;
pub type ContractAddress = Felt;
pub type ContractNonce = Felt;
pub type SierraHash = Felt;
pub type StateCommitment = Felt;
pub type StateDiffCommitment = Felt;
pub type StorageAddress = Felt;
pub type StorageValue = Felt;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StateUpdate {
    pub block_hash: BlockHash,
    pub parent_state_commitment: StateCommitment,
    pub state_commitment: StateCommitment,
    pub contract_updates: HashMap<ContractAddress, ContractUpdate>,
    pub system_contract_updates: HashMap<ContractAddress, SystemContractUpdate>,
    pub declared_cairo_classes: HashSet<ClassHash>,
    pub declared_sierra_classes: HashMap<SierraHash, CasmHash>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StateUpdateData {
    pub contract_updates: HashMap<ContractAddress, ContractUpdate>,
    pub system_contract_updates: HashMap<ContractAddress, SystemContractUpdate>,
    pub declared_cairo_classes: HashSet<ClassHash>,
    pub declared_sierra_classes: HashMap<SierraHash, CasmHash>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ContractUpdate {
    pub storage: HashMap<StorageAddress, StorageValue>,
    /// The class associated with this update as the result of either a deploy
    /// or class replacement transaction.
    pub class: Option<ContractClassUpdate>,
    pub nonce: Option<ContractNonce>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SystemContractUpdate {
    pub storage: HashMap<StorageAddress, StorageValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractClassUpdate {
    Deploy(ClassHash),
    Replace(ClassHash),
}

impl ContractUpdate {
    pub fn replaced_class(&self) -> Option<&ClassHash> {
        match &self.class {
            Some(ContractClassUpdate::Replace(hash)) => Some(hash),
            _ => None,
        }
    }

    pub fn deployed_class(&self) -> Option<&ClassHash> {
        match &self.class {
            Some(ContractClassUpdate::Deploy(hash)) => Some(hash),
            _ => None,
        }
    }
}

impl ContractClassUpdate {
    pub fn class_hash(&self) -> ClassHash {
        match self {
            ContractClassUpdate::Deploy(x) => *x,
            ContractClassUpdate::Replace(x) => *x,
        }
    }

    pub fn is_replaced(&self) -> bool {
        matches!(self, ContractClassUpdate::Replace(_))
    }
}

impl StateUpdate {
    pub fn with_block_hash(mut self, block_hash: BlockHash) -> Self {
        self.block_hash = block_hash;
        self
    }

    pub fn with_state_commitment(mut self, state_commitment: StateCommitment) -> Self {
        self.state_commitment = state_commitment;
        self
    }

    pub fn with_parent_state_commitment(
        mut self,
        parent_state_commitment: StateCommitment,
    ) -> Self {
        self.parent_state_commitment = parent_state_commitment;
        self
    }

    pub fn with_contract_nonce(mut self, contract: ContractAddress, nonce: ContractNonce) -> Self {
        self.contract_updates.entry(contract).or_default().nonce = Some(nonce);
        self
    }

    pub fn with_storage_update(
        mut self,
        contract: ContractAddress,
        key: StorageAddress,
        value: StorageValue,
    ) -> Self {
        self.contract_updates
            .entry(contract)
            .or_default()
            .storage
            .insert(key, value);
        self
    }

    pub fn with_system_storage_update(
        mut self,
        contract: ContractAddress,
        key: StorageAddress,
        value: StorageValue,
    ) -> Self {
        self.system_contract_updates
            .entry(contract)
            .or_default()
            .storage
            .insert(key, value);
        self
    }

    pub fn with_deployed_contract(mut self, contract: ContractAddress, class: ClassHash) -> Self {
        self.contract_updates.entry(contract).or_default().class =
            Some(ContractClassUpdate::Deploy(class));
        self
    }

    pub fn with_replaced_class(mut self, contract: ContractAddress, class: ClassHash) -> Self {
        self.contract_updates.entry(contract).or_default().class =
            Some(ContractClassUpdate::Replace(class));
        self
    }

    pub fn with_declared_sierra_class(mut self, sierra: SierraHash, casm: CasmHash) -> Self {
        self.declared_sierra_classes.insert(sierra, casm);
        self
    }

    pub fn with_declared_cairo_class(mut self, cairo: ClassHash) -> Self {
        self.declared_cairo_classes.insert(cairo);
        self
    }

    /// The number of individual changes in this state update.
    ///
    /// The total amount of:
    /// - system storage updates
    /// - contract storage updates
    /// - contract nonce updates
    /// - contract deployments
    /// - contract class replacements
    /// - class declarations
    pub fn change_count(&self) -> usize {
        self.declared_cairo_classes.len()
            + self.declared_sierra_classes.len()
            + self
                .system_contract_updates
                .iter()
                .map(|x| x.1.storage.len())
                .sum::<usize>()
            + self
                .contract_updates
                .iter()
                .map(|x| {
                    x.1.storage.len()
                        + x.1.class.as_ref().map(|_| 1).unwrap_or_default()
                        + x.1.nonce.as_ref().map(|_| 1).unwrap_or_default()
                })
                .sum::<usize>()
    }

    /// Returns the contract's new [nonce](ContractNonce) value if it exists in
    /// this state update.
    ///
    /// Note that this will return [Some(ContractNonce::ZERO)] for a contract
    /// that has been deployed, but without an explicit nonce update. This
    /// is consistent with expectations.
    pub fn contract_nonce(&self, contract: ContractAddress) -> Option<ContractNonce> {
        self.contract_updates.get(&contract).and_then(|x| {
            x.nonce.or_else(|| {
                x.class.as_ref().and_then(|c| match c {
                    ContractClassUpdate::Deploy(_) => {
                        // The contract has been just deployed in the pending block, so
                        // its nonce is zero.
                        Some(ContractNonce::ZERO)
                    }
                    ContractClassUpdate::Replace(_) => None,
                })
            })
        })
    }

    /// A contract's new class hash, if it was deployed or replaced in this
    /// state update.
    pub fn contract_class(&self, contract: ContractAddress) -> Option<ClassHash> {
        self.contract_updates
            .get(&contract)
            .and_then(|x| x.class.as_ref().map(|x| x.class_hash()))
    }

    /// The new storage value if it exists in this state update.
    ///
    /// Note that this will also return the default zero value for a contract
    /// that has been deployed, but without an explicit storage update.
    pub fn storage_value(
        &self,
        contract: ContractAddress,
        key: StorageAddress,
    ) -> Option<StorageValue> {
        self.contract_updates
            .get(&contract)
            .and_then(|update| {
                update
                    .storage
                    .iter()
                    .find_map(|(k, v)| (k == &key).then_some(*v))
                    .or_else(|| {
                        update.class.as_ref().and_then(|c| match c {
                            // If the contract has been deployed in pending but the key has not been
                            // set yet return the default value of zero.
                            ContractClassUpdate::Deploy(_) => Some(StorageValue::ZERO),
                            ContractClassUpdate::Replace(_) => None,
                        })
                    })
            })
            .or_else(|| {
                self.system_contract_updates
                    .get(&contract)
                    .and_then(|update| {
                        update
                            .storage
                            .iter()
                            .find_map(|(k, v)| (k == &key).then_some(*v))
                    })
            })
    }

    // pub fn compute_state_diff_commitment(&self, version: StarknetVersion) -> StateDiffCommitment {
    //     state_diff_commitment::compute(
    //         &self.contract_updates,
    //         &self.system_contract_updates,
    //         &self.declared_cairo_classes,
    //         &self.declared_sierra_classes,
    //         // version,
    //     )
    // }

    pub fn state_diff_length(&self) -> u32 {
        let mut len = 0;
        self.contract_updates.iter().for_each(|(_, update)| {
            len += update.storage.len();
            len += usize::from(update.nonce.is_some());
            len += usize::from(update.class.is_some());
        });
        self.system_contract_updates.iter().for_each(|(_, update)| {
            len += update.storage.len();
        });
        len += self.declared_cairo_classes.len() + self.declared_sierra_classes.len();
        len.try_into().expect("ptr size is 32bits")
    }
}

pub mod state_diff_commitment {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

    use starknet_types_core::{
        felt::Felt,
        hash::{Poseidon, StarkHash},
    };

    use super::{ContractUpdate, SystemContractUpdate};

    pub type BlockHash = Felt;
    pub type CasmHash = Felt;
    pub type ClassHash = Felt;
    pub type ContractAddress = Felt;
    pub type ContractNonce = Felt;
    pub type SierraHash = Felt;
    pub type StateCommitment = Felt;
    pub type StateDiffCommitment = Felt;
    pub type StorageAddress = Felt;
    pub type StorageValue = Felt;

    /// Compute the state diff commitment used in block commitment signatures.
    ///
    /// How to compute the value is documented in [this Starknet Community article](https://community.starknet.io/t/introducing-p2p-authentication-and-mismatch-resolution-in-v0-12-2/97993).
    pub fn compute(
        contract_updates: &HashMap<ContractAddress, ContractUpdate>,
        system_contract_updates: &HashMap<ContractAddress, SystemContractUpdate>,
        declared_cairo_classes: &HashSet<ClassHash>,
        declared_sierra_classes: &HashMap<SierraHash, CasmHash>,
        // version: StarknetVersion,
    ) -> Felt {
        // hasher.update(Felt::from_bytes_be_slice(b"STARKNET_STATE_DIFF0"));
        let mut data = vec![Felt::from_bytes_be_slice(b"STARKNET_STATE_DIFF0")];

        // Hash the deployed contracts.
        let deployed_contracts: BTreeMap<_, _> = contract_updates
            .iter()
            .filter_map(|(address, update)| {
                update
                    .class
                    .as_ref()
                    .map(|update| (*address, update.class_hash()))
            })
            .collect();
        data.push((deployed_contracts.len() as u64).into());
        for (address, class_hash) in deployed_contracts {
            data.push(address);
            data.push(class_hash);
        }

        // Hash the declared classes.
        let declared_classes: BTreeSet<_> = declared_sierra_classes
            .iter()
            .map(|(sierra, casm)| (*sierra, *casm))
            .collect();
        data.push((declared_classes.len() as u64).into());
        for (sierra, casm) in declared_classes {
            data.push(sierra);
            data.push(casm);
        }

        // Hash the old declared classes.
        let deprecated_declared_classes: BTreeSet<_> =
            declared_cairo_classes.iter().copied().collect();
        data.push((deprecated_declared_classes.len() as u64).into());
        for class_hash in deprecated_declared_classes {
            data.push(class_hash);
        }

        data.push(Felt::ONE);
        data.push(Felt::ZERO);

        // Hash the storage diffs.
        let storage_diffs: BTreeMap<_, _> = contract_updates
            .iter()
            .map(|(address, update)| (address, &update.storage))
            .chain(
                system_contract_updates
                    .iter()
                    .map(|(address, update)| (address, &update.storage)),
            )
            .filter_map(|(address, storage)| {
                if storage.is_empty() {
                    None
                } else {
                    let updates: BTreeMap<_, _> =
                        storage.iter().map(|(key, value)| (*key, *value)).collect();
                    Some((*address, updates))
                }
            })
            .collect();
        data.push((storage_diffs.len() as u64).into());
        for (address, updates) in storage_diffs {
            data.push(address);
            data.push((updates.len() as u64).into());
            for (key, value) in updates {
                data.push(key);
                data.push(value);
            }
        }

        // Hash the nonce updates.
        let nonces: BTreeMap<_, _> = contract_updates
            .iter()
            .filter_map(|(address, update)| update.nonce.map(|nonce| (*address, nonce)))
            .collect();
        data.push((nonces.len() as u64).into());
        for (address, nonce) in nonces {
            data.push(address);
            data.push(nonce);
        }

        Poseidon::hash_array(&data)
    }
}

//     /// Source:
//     /// https://github.com/starkware-libs/starknet-api/blob/5565e5282f5fead364a41e49c173940fd83dee00/src/block_hash/state_diff_hash_test.rs#L14
#[test]
fn test_0_13_2_state_diff_commitment() {
    let contract_updates: HashMap<_, _> = [
        (
            0u64.into(),
            ContractUpdate {
                class: Some(ContractClassUpdate::Deploy(1u64.into())),
                ..Default::default()
            },
        ),
        (
            2u64.into(),
            ContractUpdate {
                class: Some(ContractClassUpdate::Deploy(3u64.into())),
                ..Default::default()
            },
        ),
        (
            4u64.into(),
            ContractUpdate {
                storage: [(5u64.into(), 6u64.into()), (7u64.into(), 8u64.into())]
                    .iter()
                    .cloned()
                    .collect(),
                ..Default::default()
            },
        ),
        (
            9u64.into(),
            ContractUpdate {
                storage: [(10u64.into(), 11u64.into())].iter().cloned().collect(),
                ..Default::default()
            },
        ),
        (
            17u64.into(),
            ContractUpdate {
                nonce: Some(18u64.into()),
                ..Default::default()
            },
        ),
        (
            (19u64.into()),
            ContractUpdate {
                class: Some(ContractClassUpdate::Replace(20u64.into())),
                ..Default::default()
            },
        ),
    ]
    .into_iter()
    .collect();
    let declared_sierra_classes: HashMap<_, _> = [
        ((12u64.into()), (13u64.into())),
        ((14u64.into()), (15u64.into())),
    ]
    .iter()
    .cloned()
    .collect();
    let declared_cairo_classes: HashSet<_> = [(16u64.into())].iter().cloned().collect();

    let expected_hash = Felt::from_hex_unchecked(
        "0x0281f5966e49ad7dad9323826d53d1d27c0c4e6ebe5525e2e2fbca549bfa0a67",
    );

    assert_eq!(
        expected_hash,
        state_diff_commitment::compute(
            &contract_updates,
            &Default::default(),
            &declared_cairo_classes,
            &declared_sierra_classes,
        )
    );
}
