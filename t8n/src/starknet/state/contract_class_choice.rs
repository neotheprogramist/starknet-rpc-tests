use std::str::FromStr;

use starknet_devnet_types::contract_class::{Cairo0ContractClass, Cairo0Json, ContractClass};
use starknet_devnet_types::felt::Felt;
use starknet_devnet_types::traits::HashProducer;
use starknet_rs_core::types::FieldElement;
use starknet_rs_core::utils::get_selector_from_name;

use super::constants::{CAIRO_0_ACCOUNT_CONTRACT, CAIRO_1_ACCOUNT_CONTRACT_SIERRA};
use super::errors::{DevnetResult, Error};

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum AccountContractClassChoice {
    Cairo0,
    Cairo1,
}

impl AccountContractClassChoice {
    pub fn get_class_wrapper(&self) -> DevnetResult<AccountClassWrapper> {
        Ok(match self {
            AccountContractClassChoice::Cairo0 => {
                let contract_json = Cairo0Json::raw_json_from_json_str(CAIRO_0_ACCOUNT_CONTRACT)?;
                let contract_class = Cairo0ContractClass::RawJson(contract_json);
                AccountClassWrapper {
                    class_hash: contract_class.generate_hash()?,
                    contract_class: ContractClass::Cairo0(contract_class),
                }
            }
            AccountContractClassChoice::Cairo1 => {
                let contract_class = ContractClass::Cairo1(
                    ContractClass::cairo_1_from_sierra_json_str(CAIRO_1_ACCOUNT_CONTRACT_SIERRA)?,
                );
                AccountClassWrapper {
                    class_hash: contract_class.generate_hash()?,
                    contract_class,
                }
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct AccountClassWrapper {
    pub contract_class: ContractClass,
    pub class_hash: Felt,
}

impl FromStr for AccountClassWrapper {
    type Err = Error;

    fn from_str(path_candidate: &str) -> Result<Self, Self::Err> {
        // load artifact
        let contract_class = ContractClass::cairo_1_from_sierra_json_str(
            std::fs::read_to_string(path_candidate)?.as_str(),
        )?;

        // check that artifact is really account
        let execute_selector: FieldElement = get_selector_from_name("__execute__").unwrap();
        let validate_selector: FieldElement = get_selector_from_name("__validate__").unwrap();
        let mut has_execute = false;
        let mut has_validate = false;
        for entry_point in contract_class.entry_points_by_type.external.iter() {
            let selector_bytes = entry_point.selector.to_bytes_be();
            match FieldElement::from_byte_slice_be(&selector_bytes) {
                Ok(selector) if selector == execute_selector => has_execute = true,
                Ok(selector) if selector == validate_selector => has_validate = true,
                _ => (),
            }
        }
        if !has_execute || !has_validate {
            let msg = format!(
                "Not a valid Sierra account artifact; has __execute__: {has_execute}; has \
                 __validate__: {has_validate}"
            );
            return Err(Error::ContractClassLoadError(msg));
        }

        // generate the hash and return
        let contract_class = ContractClass::Cairo1(contract_class);
        let class_hash = contract_class.generate_hash()?;
        Ok(Self {
            contract_class,
            class_hash,
        })
    }
}
