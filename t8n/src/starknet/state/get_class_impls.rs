use blockifier::state::state_api::StateReader;
use starknet_devnet_types::contract_address::ContractAddress;
use starknet_devnet_types::contract_class::ContractClass;
use starknet_devnet_types::felt::ClassHash;
use starknet_rs_core::types::BlockId;

use super::errors::{DevnetResult, Error, StateError};
use super::starknet_state::CustomStateReader;
use super::Starknet;

pub fn get_class_hash_at_impl(
    starknet: &mut Starknet,
    block_id: &BlockId,
    contract_address: ContractAddress,
) -> DevnetResult<ClassHash> {
    let state = starknet.get_mut_state_at(block_id)?;
    let core_address = contract_address.try_into()?;

    let class_hash = state.get_class_hash_at(core_address)?;
    if class_hash == Default::default() {
        Err(Error::ContractNotFound)
    } else {
        Ok(class_hash.into())
    }
}

pub fn get_class_impl(
    starknet: &mut Starknet,
    block_id: &BlockId,
    class_hash: ClassHash,
) -> DevnetResult<ContractClass> {
    let state = starknet.get_mut_state_at(block_id)?;
    match state.get_rpc_contract_class(&class_hash) {
        Some(class) => Ok(class.clone()),
        None => Err(Error::StateError(StateError::NoneClassHash(class_hash))),
    }
}

pub fn get_class_at_impl(
    starknet: &mut Starknet,
    block_id: &BlockId,
    contract_address: ContractAddress,
) -> DevnetResult<ContractClass> {
    let class_hash = starknet.get_class_hash_at(block_id, contract_address)?;
    starknet.get_class(block_id, class_hash)
}
