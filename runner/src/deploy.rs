use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_crypto::FieldElement;
use starknet_signers::LocalWallet;
use utils::{
    account::single_owner::SingleOwnerAccount, contract::factory::ContractFactory,
    models::InvokeTransactionResult, provider::Provider,
};
pub async fn deploy_contract_v3<P: Provider + Send + Sync>(
    account: &SingleOwnerAccount<P, LocalWallet>,
    class_hash: FieldElement,
) -> InvokeTransactionResult {
    let factory = ContractFactory::new(class_hash, account);
    let mut salt_buffer = [0u8; 32];
    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut salt_buffer[1..]);
    let result = factory
        .deploy_v3(
            vec![],
            FieldElement::from_bytes_be(&salt_buffer).unwrap(),
            true,
        )
        .send()
        .await
        .unwrap();
    result
}
