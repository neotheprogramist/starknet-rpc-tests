use std::sync::Arc;

use super::{
    constants::{
        CAIRO_0_ACCOUNT_CONTRACT, CHARGEABLE_ACCOUNT_ADDRESS, CHARGEABLE_ACCOUNT_PRIVATE_KEY,
        CHARGEABLE_ACCOUNT_PUBLIC_KEY,
    },
    dict_state::DictState,
    errors::{DevnetResult, Error},
    starknet_state::{CustomState, StarknetState},
    traits::{Accounted, Deployed},
    utils::get_storage_var_address,
};
use blockifier::{abi::sierra_types::next_storage_key, state::state_api::StateReader};
use serde::{Deserialize, Serialize};
use starknet_api::core::PatriciaKey;
use starknet_api::hash::StarkHash;
use starknet_api::{
    core::calculate_contract_address, patricia_key, stark_felt, transaction::ContractAddressSalt,
};
use starknet_api::{hash::StarkFelt, transaction::Calldata};
use starknet_devnet_types::{
    contract_address::ContractAddress,
    contract_class::{Cairo0Json, ContractClass},
    felt::{split_biguint, ClassHash, Felt, Key},
    num_bigint::BigUint,
    rpc::state::Balance,
    traits::HashProducer,
};
/// data taken from https://github.com/0xSpaceShard/starknet-devnet/blob/fb96e0cc3c1c31fb29892ecefd2a670cf8a32b51/starknet_devnet/account.py
const ACCOUNT_CLASS_HASH_HEX_FOR_ADDRESS_COMPUTATION: &str =
    "0x3FCBF77B28C96F4F2FB5BD2D176AB083A12A5E123ADEB0DE955D7EE228C9854";

pub enum FeeToken {
    ETH,
    STRK,
}
#[derive(Deserialize, Serialize)]
pub struct PartialUserAccount {
    pub public_key: Key,
    pub account_address: ContractAddress,
    pub initial_balance: Balance,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserAccount {
    pub public_key: Key,
    pub account_address: ContractAddress,
    pub initial_balance: Balance,
    pub class_hash: ClassHash,
    pub contract_class: ContractClass,
    pub eth_fee_token_address: ContractAddress,
    pub strk_fee_token_address: ContractAddress,
}

#[derive(Clone, Debug)]
pub struct Account {
    pub public_key: Key,
    pub private_key: Key,
    pub account_address: ContractAddress,
    pub initial_balance: Balance,
    pub class_hash: ClassHash,
    pub(crate) contract_class: ContractClass,
    pub(crate) eth_fee_token_address: ContractAddress,
    pub(crate) strk_fee_token_address: ContractAddress,
}

impl UserAccount {
    pub fn new(
        initial_balance: Balance,
        public_key: Key,
        account_address: ContractAddress,
        class_hash: ClassHash,
        contract_class: ContractClass,
        eth_fee_token_address: ContractAddress,
        strk_fee_token_address: ContractAddress,
    ) -> DevnetResult<Self> {
        Ok(Self {
            initial_balance,
            public_key,
            account_address,
            class_hash,
            contract_class,
            eth_fee_token_address,
            strk_fee_token_address,
        })
    }
}

impl Account {
    pub(crate) fn new_chargeable(
        eth_fee_token_address: ContractAddress,
        strk_fee_token_address: ContractAddress,
    ) -> DevnetResult<Self> {
        let account_contract_class = Cairo0Json::raw_json_from_json_str(CAIRO_0_ACCOUNT_CONTRACT)?;
        let class_hash = account_contract_class.generate_hash()?;

        // insanely big - should practically never run out of funds
        let initial_balance = Balance::from(u128::MAX);
        Ok(Self {
            public_key: Key::from_prefixed_hex_str(CHARGEABLE_ACCOUNT_PUBLIC_KEY)?,
            private_key: Key::from_prefixed_hex_str(CHARGEABLE_ACCOUNT_PRIVATE_KEY)?,
            account_address: ContractAddress::new(Felt::from_prefixed_hex_str(
                CHARGEABLE_ACCOUNT_ADDRESS,
            )?)?,
            initial_balance,
            class_hash,
            contract_class: account_contract_class.into(),
            eth_fee_token_address,
            strk_fee_token_address,
        })
    }

    pub(crate) fn new(
        initial_balance: Balance,
        public_key: Key,
        private_key: Key,
        class_hash: ClassHash,
        contract_class: ContractClass,
        eth_fee_token_address: ContractAddress,
        strk_fee_token_address: ContractAddress,
    ) -> DevnetResult<Self> {
        Ok(Self {
            initial_balance,
            public_key,
            private_key,
            class_hash,
            contract_class,
            account_address: Account::compute_account_address(&public_key)?,
            eth_fee_token_address,
            strk_fee_token_address,
        })
    }

    fn compute_account_address(public_key: &Key) -> DevnetResult<ContractAddress> {
        let account_address = calculate_contract_address(
            ContractAddressSalt(stark_felt!(20u32)),
            Felt::from_prefixed_hex_str(ACCOUNT_CLASS_HASH_HEX_FOR_ADDRESS_COMPUTATION)?.into(),
            &Calldata(Arc::new(vec![(*public_key).into()])),
            starknet_api::core::ContractAddress(patricia_key!(0u32)),
        )
        .map_err(Error::StarknetApiError)?;

        Ok(ContractAddress::from(account_address))
    }
}

impl Deployed for Account {
    fn deploy(&self, state: &mut StarknetState) -> DevnetResult<()> {
        self.declare_if_undeclared(state, self.class_hash, &self.contract_class)?;

        state.predeploy_contract(self.account_address, self.class_hash)?;

        // set public key directly in the most underlying state
        let public_key_storage_var = get_storage_var_address("Account_public_key", &[])?;
        state.state.state.set_storage_at(
            self.account_address.try_into()?,
            public_key_storage_var.try_into()?,
            self.public_key.into(),
        )?;

        // set balance directly in the most underlying state
        self.set_initial_balance(&mut state.state.state)?;

        Ok(())
    }

    fn get_address(&self) -> ContractAddress {
        self.account_address
    }
}

impl Deployed for UserAccount {
    fn deploy(&self, state: &mut StarknetState) -> DevnetResult<()> {
        self.declare_if_undeclared(state, self.class_hash, &self.contract_class)?;

        state.predeploy_contract(self.account_address, self.class_hash)?;

        // set public key directly in the most underlying state
        let public_key_storage_var = get_storage_var_address("Account_public_key", &[])?;
        state.state.state.set_storage_at(
            self.account_address.try_into()?,
            public_key_storage_var.try_into()?,
            self.public_key.into(),
        )?;

        // set balance directly in the most underlying state
        self.set_initial_balance(&mut state.state.state)?;

        Ok(())
    }

    fn get_address(&self) -> ContractAddress {
        self.account_address
    }
}

impl Accounted for Account {
    fn set_initial_balance(&self, state: &mut DictState) -> DevnetResult<()> {
        let storage_var_address_low =
            get_storage_var_address("ERC20_balances", &[Felt::from(self.account_address)])?;
        let storage_var_address_high = next_storage_key(&storage_var_address_low.try_into()?)?;

        let (high, low) = split_biguint(self.initial_balance.clone())?;

        for fee_token_address in [self.eth_fee_token_address, self.strk_fee_token_address] {
            state.set_storage_at(
                fee_token_address.try_into()?,
                storage_var_address_low.try_into()?,
                low.into(),
            )?;

            state.set_storage_at(
                fee_token_address.try_into()?,
                storage_var_address_high,
                high.into(),
            )?;
        }

        Ok(())
    }

    fn get_balance(&self, state: &mut impl StateReader, token: FeeToken) -> DevnetResult<Balance> {
        let fee_token_address = match token {
            FeeToken::ETH => self.eth_fee_token_address,
            FeeToken::STRK => self.strk_fee_token_address,
        };
        let (low, high) = state.get_fee_token_balance(
            self.account_address.try_into()?,
            fee_token_address.try_into()?,
        )?;
        let low: BigUint = Felt::from(low).into();
        let high: BigUint = Felt::from(high).into();
        Ok(low + (high << 128))
    }
}

impl Accounted for UserAccount {
    fn set_initial_balance(&self, state: &mut DictState) -> DevnetResult<()> {
        let storage_var_address_low =
            get_storage_var_address("ERC20_balances", &[Felt::from(self.account_address)])?;
        let storage_var_address_high = next_storage_key(&storage_var_address_low.try_into()?)?;

        let (high, low) = split_biguint(self.initial_balance.clone())?;

        for fee_token_address in [self.eth_fee_token_address, self.strk_fee_token_address] {
            state.set_storage_at(
                fee_token_address.try_into()?,
                storage_var_address_low.try_into()?,
                low.into(),
            )?;

            state.set_storage_at(
                fee_token_address.try_into()?,
                storage_var_address_high,
                high.into(),
            )?;
        }

        Ok(())
    }

    fn get_balance(&self, state: &mut impl StateReader, token: FeeToken) -> DevnetResult<Balance> {
        let fee_token_address = match token {
            FeeToken::ETH => self.eth_fee_token_address,
            FeeToken::STRK => self.strk_fee_token_address,
        };
        let (low, high) = state.get_fee_token_balance(
            self.account_address.try_into()?,
            fee_token_address.try_into()?,
        )?;
        let low: BigUint = Felt::from(low).into();
        let high: BigUint = Felt::from(high).into();
        Ok(low + (high << 128))
    }
}
