use blockifier::state::state_api::StateReader;
use starknet_devnet_types::contract_address::ContractAddress;
use starknet_devnet_types::contract_class::{Cairo0Json, ContractClass};
use starknet_devnet_types::felt::{ClassHash, Felt};
use starknet_devnet_types::rpc::state::Balance;

use super::account::FeeToken;
use super::dict_state::DictState;
use super::errors::DevnetResult;
use super::starknet_state::{CustomState, StarknetState};
use super::traits::{Accounted, Deployed};

pub(crate) struct SystemContract {
    class_hash: ClassHash,
    address: ContractAddress,
    contract_class: ContractClass,
}

impl SystemContract {
    pub(crate) fn new_cairo0(
        class_hash: &str,
        address: &str,
        contract_class_json_str: &str,
    ) -> DevnetResult<Self> {
        Ok(Self {
            class_hash: Felt::from_prefixed_hex_str(class_hash)?,
            address: ContractAddress::new(Felt::from_prefixed_hex_str(address)?)?,
            contract_class: Cairo0Json::raw_json_from_json_str(contract_class_json_str)?.into(),
        })
    }

    pub(crate) fn new_cairo1(
        class_hash: &str,
        address: &str,
        contract_class_json_str: &str,
    ) -> DevnetResult<Self> {
        Ok(Self {
            class_hash: Felt::from_prefixed_hex_str(class_hash)?,
            address: ContractAddress::new(Felt::from_prefixed_hex_str(address)?)?,
            contract_class: ContractClass::cairo_1_from_sierra_json_str(contract_class_json_str)?
                .into(),
        })
    }
}

impl Deployed for SystemContract {
    fn deploy(&self, state: &mut StarknetState) -> DevnetResult<()> {
        self.declare_if_undeclared(state, self.class_hash, &self.contract_class)?;
        state.predeploy_contract(self.address, self.class_hash)?;
        Ok(())
    }

    fn get_address(&self) -> ContractAddress {
        self.address
    }
}

impl Accounted for SystemContract {
    fn set_initial_balance(&self, _state: &mut DictState) -> DevnetResult<()> {
        Ok(())
    }

    fn get_balance(
        &self,
        _state: &mut impl StateReader,
        _token: FeeToken,
    ) -> DevnetResult<Balance> {
        Ok(Balance::default())
    }
}
