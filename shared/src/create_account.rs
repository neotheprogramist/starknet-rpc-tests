use crate::mint::{mint, MintParams};
use rand::Rng;
use starknet_crypto::{pedersen_hash, FieldElement};
use starknet_signers::SigningKey;
use url::Url;

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

pub async fn create_account(base_url: &Url) {
    let key = SigningKey::from_random();

    let salt = SigningKey::from_random().secret_scalar();

    let class_hash = FieldElement::from_hex_be(
        "0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
    )
    .unwrap();

    let account_configuration = AccountConfiguration {
        class_hash,
        salt,
        public_key: key.verifying_key().scalar(),
    };

    let deployed_address = get_contract_address(&account_configuration);

    let mut rng = rand::thread_rng();
    let mint_params = MintParams {
        address: deployed_address,
        amount: rng.gen_range(u64::MAX as u128..u128::MAX),
    };

    let mint_response = match mint(&mint_params, base_url).await {
        Ok(mint_response) => mint_response,
        Err(e) => panic!("Error minting: {:?}", e),
    };
    println!("Mint Response: {:?}", mint_response);
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
