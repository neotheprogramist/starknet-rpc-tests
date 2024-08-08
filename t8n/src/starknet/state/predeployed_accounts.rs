use starknet_devnet_types::{
    contract_address::ContractAddress,
    contract_class::ContractClass,
    felt::{ClassHash, Key},
    rpc::state::Balance,
};
use starknet_rs_core::types::FieldElement;
use starknet_rs_signers::SigningKey;

use super::{
    account::Account, errors::DevnetResult, traits::AccountGenerator,
    utils::random_number_generator::generate_u128_random_numbers,
};

#[derive(Default)]
pub struct PredeployedAccounts {
    pub seed: u32,
    pub initial_balance: Balance,
    pub eth_fee_token_address: ContractAddress,
    pub strk_fee_token_address: ContractAddress,
    pub accounts: Vec<Account>,
}

impl PredeployedAccounts {
    pub(crate) fn new(
        seed: u32,
        initial_balance: Balance,
        eth_fee_token_address: ContractAddress,
        strk_fee_token_address: ContractAddress,
    ) -> Self {
        Self {
            seed,
            initial_balance,
            eth_fee_token_address,
            strk_fee_token_address,
            accounts: Vec::new(),
        }
    }
}

impl PredeployedAccounts {
    fn generate_private_keys(&self, number_of_accounts: u8) -> Vec<Key> {
        let random_numbers = generate_u128_random_numbers(self.seed, number_of_accounts);
        random_numbers
            .into_iter()
            .map(Key::from)
            .collect::<Vec<Key>>()
    }

    fn generate_public_key(&self, private_key: &Key) -> Key {
        let private_key_field_element = FieldElement::from(*private_key);

        Key::from(
            SigningKey::from_secret_scalar(private_key_field_element)
                .verifying_key()
                .scalar(),
        )
    }

    pub fn get_accounts(&self) -> &Vec<Account> {
        &self.accounts
    }
}

impl AccountGenerator for PredeployedAccounts {
    type Acc = Account;

    fn generate_accounts(
        &mut self,
        number_of_accounts: u8,
        class_hash: ClassHash,
        contract_class: &ContractClass,
    ) -> DevnetResult<&Vec<Self::Acc>> {
        let private_keys = self.generate_private_keys(number_of_accounts);

        for private_key in private_keys {
            let account = Account::new(
                self.initial_balance.clone(),
                self.generate_public_key(&private_key),
                private_key,
                class_hash,
                contract_class.clone(),
                self.eth_fee_token_address,
                self.strk_fee_token_address,
            )?;
            self.accounts.push(account);
        }

        Ok(&self.accounts)
    }
}
