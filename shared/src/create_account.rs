use starknet_crypto::{pedersen_hash, FieldElement};
use starknet_signers::SigningKey;

// Cairo string of "STARKNET_CONTRACT_ADDRESS"
const CONTRACT_ADDRESS_PREFIX: FieldElement = FieldElement::from_mont([
    3829237882463328880,
    17289941567720117366,
    8635008616843941496,
    533439743893157637,
]);

#[derive(Debug)]
pub struct AccountConfiguration {
    pub class_hash: FieldElement,
    pub salt: FieldElement,
    pub public_key: FieldElement,
}

pub fn create_account() {
    let key = SigningKey::from_random();
    println!("Private Key: {:?}", &key.secret_scalar());
    println!("Public Key: {:?}", &key.verifying_key().scalar());

    let salt = SigningKey::from_random().secret_scalar();
    println!("Salt: {:?}", &salt);

    let class_hash = FieldElement::from_hex_be(
        "0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
    )
    .unwrap();
    println!("Class Hash: {:?}", &class_hash);

    let account_configuration = AccountConfiguration {
        class_hash,
        salt,
        public_key: key.verifying_key().scalar(),
    };
    println!("Account Configuration: {:?}", &account_configuration);
    let deployed_address = get_contract_address(&account_configuration);
    println!("Deployed Address: {:?}", &deployed_address);
}

pub fn get_contract_address(account_configuration: &AccountConfiguration) -> FieldElement {
    compute_hash_on_elements(&[
        CONTRACT_ADDRESS_PREFIX,
        FieldElement::ZERO,
        account_configuration.salt,
        account_configuration.class_hash,
        compute_hash_on_elements(&[account_configuration.public_key]),
    ])
}

pub fn compute_hash_on_elements(data: &[FieldElement]) -> FieldElement {
    let mut current_hash = FieldElement::ZERO;

    for item in data.iter() {
        current_hash = pedersen_hash(&current_hash, item);
    }

    let data_len = FieldElement::from(data.len());
    pedersen_hash(&current_hash, &data_len)
}
