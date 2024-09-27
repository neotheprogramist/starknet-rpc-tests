use blockifier::{
    execution::contract_class::{ContractClass, ContractClassHelper},
    state::{
        cached_state::StorageEntry,
        errors::StateError,
        state_api::{StateReader, StateResult},
    },
};
use serde::{de, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use starknet_api::{
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    hash::{StarkFelt, StarkHash},
    state::StorageKey,
};
use std::collections::HashMap;

use super::defaulter::StarknetDefaulter;

/// A simple implementation of `StateReader` using `HashMap`s as storage.
/// Copied from blockifier test_utils, added `impl State`
#[derive(Debug, Default, Clone)]
pub struct DictState {
    pub storage_view: HashMap<StorageEntry, StarkFelt>,
    pub address_to_nonce: HashMap<ContractAddress, Nonce>,
    pub address_to_class_hash: HashMap<ContractAddress, ClassHash>,
    pub class_hash_to_class: HashMap<ClassHash, ContractClass>,
    pub class_hash_to_compiled_class_hash: HashMap<ClassHash, CompiledClassHash>,
    defaulter: StarknetDefaulter,
}

struct MyStorageEntry(StorageEntry);

impl std::str::FromStr for MyStorageEntry {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err("Invalid input string".to_string());
        }

        let contract_address_value = parts[0].trim();
        let storage_key_value = parts[1].trim();

        let contract_address: StarkHash = StarkHash::try_from(contract_address_value).unwrap();
        let storage_key: StarkHash = StarkHash::try_from(storage_key_value).unwrap();

        let storage_entry: StorageEntry = (
            ContractAddress::try_from(contract_address).unwrap(),
            StorageKey::try_from(storage_key).unwrap(),
        );
        Ok(MyStorageEntry(storage_entry))
    }
}

impl From<MyStorageEntry> for StorageEntry {
    fn from(entry: MyStorageEntry) -> Self {
        entry.0
    }
}

fn convert_hash_map_helper(
    input: HashMap<ClassHash, ContractClassHelper>,
) -> HashMap<ClassHash, ContractClass> {
    input.into_iter().map(|(k, v)| (k, v.into())).collect()
}

impl<'de> Deserialize<'de> for DictState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DictStateHelper {
            storage_view: HashMap<String, String>, // String keys and values
            address_to_nonce: HashMap<ContractAddress, Nonce>,
            address_to_class_hash: HashMap<ContractAddress, ClassHash>,
            class_hash_to_class: HashMap<ClassHash, ContractClassHelper>,
            class_hash_to_compiled_class_hash: HashMap<ClassHash, CompiledClassHash>,
        }

        let helper = DictStateHelper::deserialize(deserializer)?;

        let storage_view: HashMap<StorageEntry, StarkFelt> = helper
            .storage_view
            .into_iter()
            .map(|(k, v)| {
                let parts: Vec<&str> = k.split(", ").collect();
                if parts.len() != 2 {
                    return Err(de::Error::custom("Invalid storage_view key format"));
                }
                let contract_address_str = parts[0].replace("contract_address: ", "");
                let storage_key_str = parts[1].replace("storage_key: ", "");

                let contract_address: StarkHash =
                    StarkHash::try_from(contract_address_str.as_str())
                        .map_err(de::Error::custom)?;
                let storage_key: StarkHash =
                    StarkHash::try_from(storage_key_str.as_str()).map_err(de::Error::custom)?;

                let storage_entry: StorageEntry = (
                    ContractAddress::try_from(contract_address).map_err(de::Error::custom)?,
                    StorageKey::try_from(storage_key).map_err(de::Error::custom)?,
                );
                let stark_felt = StarkFelt::try_from(v.as_str()).map_err(de::Error::custom)?;

                Ok((storage_entry, stark_felt))
            })
            .collect::<Result<HashMap<StorageEntry, StarkFelt>, D::Error>>()?;

        let class_hash_to_class = convert_hash_map_helper(helper.class_hash_to_class);

        Ok(DictState {
            storage_view,
            address_to_nonce: helper.address_to_nonce,
            address_to_class_hash: helper.address_to_class_hash,
            class_hash_to_class,
            class_hash_to_compiled_class_hash: helper.class_hash_to_compiled_class_hash,
            defaulter: StarknetDefaulter::default(),
        })
    }
}

impl Serialize for DictState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(6))?;

        let storage_view: HashMap<String, &StarkFelt> = self
            .storage_view
            .iter()
            .map(|(k, v)| {
                let (contract_address, storage_key) = k;
                let contract_address_str = format!("contract_address: {}", *contract_address.0);
                let storage_key_str = format!("storage_key: {}", *storage_key.0);
                (format!("{}, {}", contract_address_str, storage_key_str), v)
            })
            .collect();
        state.serialize_entry("storage_view", &storage_view)?;

        state.serialize_entry("address_to_nonce", &self.address_to_nonce)?;

        state.serialize_entry("address_to_class_hash", &self.address_to_class_hash)?;

        state.serialize_entry("class_hash_to_class", &self.class_hash_to_class)?;

        state.serialize_entry(
            "class_hash_to_compiled_class_hash",
            &self.class_hash_to_compiled_class_hash,
        )?;

        state.end()
    }
}

impl DictState {
    pub fn new(defaulter: StarknetDefaulter) -> Self {
        Self {
            defaulter,
            ..Self::default()
        }
    }
}

impl StateReader for DictState {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        let contract_storage_key = (contract_address, key);
        match self.storage_view.get(&contract_storage_key) {
            Some(value) => Ok(*value),
            None => self.defaulter.get_storage_at(contract_address, key),
        }
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        match self.address_to_nonce.get(&contract_address) {
            Some(value) => Ok(*value),
            None => self.defaulter.get_nonce_at(contract_address),
        }
    }

    fn get_compiled_contract_class(&mut self, class_hash: ClassHash) -> StateResult<ContractClass> {
        match self.class_hash_to_class.get(&class_hash) {
            Some(contract_class) => Ok(contract_class.clone()),
            None => self.defaulter.get_compiled_contract_class(class_hash),
        }
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        match self.address_to_class_hash.get(&contract_address) {
            Some(class_hash) => Ok(*class_hash),
            None => self.defaulter.get_class_hash_at(contract_address),
        }
    }

    fn get_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
    ) -> StateResult<starknet_api::core::CompiledClassHash> {
        // can't ask origin for this - insufficient API - probably not important
        let compiled_class_hash = self
            .class_hash_to_compiled_class_hash
            .get(&class_hash)
            .copied()
            .unwrap_or_default();
        Ok(compiled_class_hash)
    }
}

// Basing the methods on blockifier's `State` interface, without those that would never be used
// (add_visited_pcs, to_state_diff)
impl DictState {
    pub fn set_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
        value: StarkFelt,
    ) -> std::result::Result<(), blockifier::state::errors::StateError> {
        self.storage_view.insert((contract_address, key), value);
        Ok(())
    }

    pub fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        let current_nonce = self.get_nonce_at(contract_address)?;
        let current_nonce_as_u64 = usize::try_from(current_nonce.0)? as u64;
        let next_nonce_val = 1_u64 + current_nonce_as_u64;
        let next_nonce = Nonce(StarkFelt::from(next_nonce_val));
        self.address_to_nonce.insert(contract_address, next_nonce);

        Ok(())
    }

    pub fn set_nonce(
        &mut self,
        contract_address: ContractAddress,
        nonce: Nonce,
    ) -> StateResult<()> {
        self.address_to_nonce.insert(contract_address, nonce);
        Ok(())
    }

    pub fn set_class_hash_at(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) -> StateResult<()> {
        if contract_address == ContractAddress::default() {
            return Err(StateError::OutOfRangeContractAddress);
        }

        self.address_to_class_hash
            .insert(contract_address, class_hash);
        Ok(())
    }

    pub fn set_contract_class(
        &mut self,
        class_hash: ClassHash,
        contract_class: ContractClass,
    ) -> StateResult<()> {
        self.class_hash_to_class.insert(class_hash, contract_class);
        Ok(())
    }

    pub fn set_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
        compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        self.class_hash_to_compiled_class_hash
            .insert(class_hash, compiled_class_hash);
        Ok(())
    }
}
