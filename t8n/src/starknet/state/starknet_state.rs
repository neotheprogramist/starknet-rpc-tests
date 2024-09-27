use std::collections::HashMap;

use super::errors::{DevnetResult, Error};
use super::utils::casm_hash;
use super::{
    defaulter::StarknetDefaulter, dict_state::DictState, state_diff::StateDiff, types::ClassHash,
};
use blockifier::state::state_api::StateReader;
use blockifier::state::{
    cached_state::{CachedState, GlobalContractCache, GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST},
    state_api::State,
};
use serde::{Deserialize, Serialize};
use starknet_api::block::{BlockHeader, BlockNumber};
use starknet_api::core::Nonce;
use starknet_api::{core::CompiledClassHash, hash::StarkFelt};
use starknet_devnet_types::contract_address::ContractAddress;
use starknet_devnet_types::contract_class::ContractClass;
use starknet_devnet_types::felt::Felt;

pub trait CustomStateReader {
    fn is_contract_deployed(&mut self, contract_address: ContractAddress) -> DevnetResult<bool>;
    fn is_contract_declared(&mut self, class_hash: ClassHash) -> bool;
    /// sierra for cairo1, only artifact for cairo0
    fn get_rpc_contract_class(&self, class_hash: &ClassHash) -> Option<&ContractClass>;
}

pub trait CustomState {
    fn predeclare_contract_class(
        &mut self,
        class_hash: ClassHash,
        contract_class: ContractClass,
    ) -> DevnetResult<()>;
    fn declare_contract_class(
        &mut self,
        class_hash: ClassHash,
        contract_class: ContractClass,
    ) -> DevnetResult<()>;
    fn predeploy_contract(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) -> DevnetResult<()>;
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
/// Utility structure that makes it easier to calculate state diff later on
pub struct CommittedClassStorage {
    staging: HashMap<ClassHash, ContractClass>,
    committed: HashMap<ClassHash, ContractClass>,
}

impl CommittedClassStorage {
    pub fn insert(&mut self, class_hash: ClassHash, contract_class: ContractClass) {
        self.staging.insert(class_hash, contract_class);
    }

    pub fn commit(&mut self) -> HashMap<ClassHash, ContractClass> {
        let diff = self.staging.clone();
        self.committed.extend(self.staging.drain());
        diff
    }

    /// Skips the staging phase
    fn insert_and_commit(&mut self, class_hash: ClassHash, contract_class: ContractClass) {
        assert!(self.staging.is_empty());
        self.insert(class_hash, contract_class);
        self.commit();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateWithBlockNumber {
    pub state: StarknetState,
    pub block_number: BlockNumber,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateWithBlock {
    pub state: StarknetState,
    pub blocks: Block,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StarknetState {
    pub state: CachedState<DictState>,
    pub rpc_contract_classes: CommittedClassStorage,
    /// - initially `None`
    /// - indicates the state hasn't yet been cloned for old-state preservation purpose
    pub historic_state: Option<DictState>,
}
impl Default for StarknetState {
    fn default() -> Self {
        Self {
            state: CachedState::new(
                Default::default(),
                GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
            ),
            rpc_contract_classes: Default::default(),
            historic_state: Default::default(),
        }
    }
}

impl StarknetState {
    pub fn new(defaulter: StarknetDefaulter) -> Self {
        Self {
            state: CachedState::new(
                DictState::new(defaulter),
                GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
            ),
            rpc_contract_classes: Default::default(),
            historic_state: Default::default(),
        }
    }

    pub fn clone_rpc_contract_classes(&self) -> CommittedClassStorage {
        self.rpc_contract_classes.clone()
    }

    pub fn commit_with_diff(&mut self) -> DevnetResult<StateDiff> {
        let diff = StateDiff::generate(&mut self.state, &mut self.rpc_contract_classes)?;
        let new_historic = self.expand_historic(diff.clone())?;
        self.state = CachedState::new(
            new_historic.clone(),
            GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
        );
        Ok(diff)
    }

    pub fn diff_trace(&mut self) -> DevnetResult<StateDiff> {
        let mut transactional_rpc_contract_classes = self.clone_rpc_contract_classes();
        let mut transactional_state = CachedState::new(
            CachedState::create_transactional(&mut self.state),
            GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
        );
        let diff = StateDiff::generate(
            &mut transactional_state,
            &mut transactional_rpc_contract_classes,
        )?;
        Ok(diff)
    }

    pub fn to_state_diff(&mut self) -> DevnetResult<StateDiff> {
        let diff = StateDiff::generate(&mut self.state, &mut self.rpc_contract_classes)?;
        // let new_historic = self.expand_historic(diff.clone())?;
        // self.state = CachedState::new(
        //     new_historic.clone(),
        //     GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
        // );
        Ok(diff)
    }

    pub fn assert_contract_deployed(
        &mut self,
        contract_address: ContractAddress,
    ) -> DevnetResult<()> {
        if !self.is_contract_deployed(contract_address)? {
            return Err(Error::ContractNotFound);
        }
        Ok(())
    }

    /// Expands the internal historic state copy and returns a reference to it
    fn expand_historic(&mut self, state_diff: StateDiff) -> DevnetResult<&DictState> {
        let mut historic_state = self.state.state.clone();

        for (address, class_hash) in state_diff.address_to_class_hash {
            historic_state.set_class_hash_at(address.try_into()?, class_hash.into())?;
        }
        for (class_hash, casm_hash) in state_diff.class_hash_to_compiled_class_hash {
            historic_state.set_compiled_class_hash(class_hash.into(), casm_hash.into())?;
        }
        for (address, nonce) in state_diff.address_to_nonce {
            // assuming that historic_state.get_nonce(address) == _nonce - 1
            historic_state.set_nonce(address.try_into()?, Nonce(nonce.into()))?;
        }
        for (address, storage_updates) in state_diff.storage_updates {
            let core_address = address.try_into()?;
            for (key, value) in storage_updates {
                historic_state.set_storage_at(core_address, key.try_into()?, value.into())?;
            }
        }
        for class_hash in state_diff.cairo_0_declared_contracts {
            let compiled_class = self.get_compiled_contract_class(class_hash.into())?;
            historic_state.set_contract_class(class_hash.into(), compiled_class)?;
        }
        for class_hash in state_diff.declared_contracts {
            let compiled_class = self.get_compiled_contract_class(class_hash.into())?;
            historic_state.set_contract_class(class_hash.into(), compiled_class)?;
        }
        self.historic_state = Some(historic_state);
        Ok(self.historic_state.as_ref().unwrap())
    }

    pub fn clone_historic(&self) -> Self {
        let historic_state = self.historic_state.as_ref().unwrap().clone();
        Self {
            state: CachedState::new(
                historic_state,
                GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
            ),
            rpc_contract_classes: self.rpc_contract_classes.clone(),
            historic_state: Some(self.historic_state.as_ref().unwrap().clone()),
        }
    }
}

impl State for StarknetState {
    fn set_storage_at(
        &mut self,
        contract_address: starknet_api::core::ContractAddress,
        key: starknet_api::state::StorageKey,
        value: starknet_api::hash::StarkFelt,
    ) -> std::result::Result<(), blockifier::state::errors::StateError> {
        self.state.set_storage_at(contract_address, key, value)
    }

    fn increment_nonce(
        &mut self,
        contract_address: starknet_api::core::ContractAddress,
    ) -> blockifier::state::state_api::StateResult<()> {
        self.state.increment_nonce(contract_address)
    }

    fn set_class_hash_at(
        &mut self,
        contract_address: starknet_api::core::ContractAddress,
        class_hash: starknet_api::core::ClassHash,
    ) -> blockifier::state::state_api::StateResult<()> {
        self.state.set_class_hash_at(contract_address, class_hash)
    }

    fn set_contract_class(
        &mut self,
        class_hash: starknet_api::core::ClassHash,
        contract_class: blockifier::execution::contract_class::ContractClass,
    ) -> blockifier::state::state_api::StateResult<()> {
        self.state.set_contract_class(class_hash, contract_class)
    }

    fn set_compiled_class_hash(
        &mut self,
        class_hash: starknet_api::core::ClassHash,
        compiled_class_hash: starknet_api::core::CompiledClassHash,
    ) -> blockifier::state::state_api::StateResult<()> {
        self.state
            .set_compiled_class_hash(class_hash, compiled_class_hash)
    }

    fn to_state_diff(&mut self) -> blockifier::state::cached_state::CommitmentStateDiff {
        self.state.to_state_diff()
    }

    fn add_visited_pcs(
        &mut self,
        class_hash: starknet_api::core::ClassHash,
        pcs: &std::collections::HashSet<usize>,
    ) {
        self.state.add_visited_pcs(class_hash, pcs)
    }
}

impl blockifier::state::state_api::StateReader for StarknetState {
    fn get_storage_at(
        &mut self,
        contract_address: starknet_api::core::ContractAddress,
        key: starknet_api::state::StorageKey,
    ) -> blockifier::state::state_api::StateResult<starknet_api::hash::StarkFelt> {
        self.state.get_storage_at(contract_address, key)
    }

    fn get_nonce_at(
        &mut self,
        contract_address: starknet_api::core::ContractAddress,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::Nonce> {
        self.state.get_nonce_at(contract_address)
    }

    fn get_class_hash_at(
        &mut self,
        contract_address: starknet_api::core::ContractAddress,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::ClassHash> {
        self.state.get_class_hash_at(contract_address)
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: starknet_api::core::ClassHash,
    ) -> blockifier::state::state_api::StateResult<
        blockifier::execution::contract_class::ContractClass,
    > {
        self.state.get_compiled_contract_class(class_hash)
    }

    fn get_compiled_class_hash(
        &mut self,
        class_hash: starknet_api::core::ClassHash,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::CompiledClassHash> {
        self.state.get_compiled_class_hash(class_hash)
    }
}

impl CustomStateReader for StarknetState {
    fn is_contract_deployed(&mut self, contract_address: ContractAddress) -> DevnetResult<bool> {
        let api_address = contract_address.try_into()?;
        let starknet_api::core::ClassHash(class_hash) = self.get_class_hash_at(api_address)?;
        Ok(class_hash != StarkFelt::ZERO)
    }

    fn is_contract_declared(&mut self, class_hash: ClassHash) -> bool {
        // get_compiled_contract_class is important if forking; checking hash is impossible via
        // JSON-RPC
        self.get_compiled_class_hash(class_hash.into())
            .is_ok_and(|CompiledClassHash(class_hash)| class_hash != StarkFelt::ZERO)
            || self.get_compiled_contract_class(class_hash.into()).is_ok()
    }

    fn get_rpc_contract_class(&self, class_hash: &ClassHash) -> Option<&ContractClass> {
        self.rpc_contract_classes.committed.get(class_hash)
    }
}

impl CustomState for StarknetState {
    /// writes directly to the most underlying state, skipping cache
    fn predeclare_contract_class(
        &mut self,
        class_hash: ClassHash,
        contract_class: ContractClass,
    ) -> DevnetResult<()> {
        let compiled_class = contract_class.clone().try_into()?;

        if let ContractClass::Cairo1(cairo_lang_contract_class) = &contract_class {
            let casm_json =
                usc::compile_contract(serde_json::to_value(cairo_lang_contract_class).map_err(
                    |err| Error::SerializationError {
                        origin: err.to_string(),
                    },
                )?)
                .map_err(|_| Error::SierraCompilationError)?;

            let casm_hash = Felt::from(casm_hash(casm_json)?);

            self.state
                .state
                .set_compiled_class_hash(class_hash.into(), casm_hash.into())?;
        };

        self.state
            .state
            .set_contract_class(class_hash.into(), compiled_class)?;
        self.rpc_contract_classes
            .insert_and_commit(class_hash, contract_class);
        Ok(())
    }

    fn declare_contract_class(
        &mut self,
        class_hash: ClassHash,
        contract_class: ContractClass,
    ) -> DevnetResult<()> {
        let compiled_class = contract_class.clone().try_into()?;

        if let ContractClass::Cairo1(cairo_lang_contract_class) = &contract_class {
            let casm_json =
                usc::compile_contract(serde_json::to_value(cairo_lang_contract_class).map_err(
                    |err| Error::SerializationError {
                        origin: err.to_string(),
                    },
                )?)
                .map_err(|_| Error::SierraCompilationError)?;

            let casm_hash = Felt::from(casm_hash(casm_json)?);
            self.set_compiled_class_hash(class_hash.into(), casm_hash.into())?;
        };

        self.set_contract_class(class_hash.into(), compiled_class)?;
        self.rpc_contract_classes.insert(class_hash, contract_class);
        Ok(())
    }

    fn predeploy_contract(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) -> DevnetResult<()> {
        self.state
            .state
            .set_class_hash_at(contract_address.try_into()?, class_hash.into())
            .map_err(|e| e.into())
    }
}
