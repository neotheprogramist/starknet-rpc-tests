use crypto_utils::curve::signer::compute_hash_on_elements;
use starknet_types_core::felt::Felt;

use crate::v5::rpc::{
    accounts::{
        account::normalize_address,
        creation::create::AccountType,
        errors::CreationError,
        factory::{open_zeppelin::OpenZeppelinAccountFactory, AccountFactory},
    },
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::{key_pair::SigningKey, local_wallet::LocalWallet},
};

use super::structs::WaitForTx;

// Cairo string of "STARKNET_CONTRACT_ADDRESS"
const CONTRACT_ADDRESS_PREFIX: Felt = Felt::from_raw([
    3829237882463328880,
    17289941567720117366,
    8635008616843941496,
    533439743893157637,
]);

/// Computes the target contract address of a "native" contract deployment. Use
/// `get_udc_deployed_address` instead if you want to compute the target address for deployments
/// through the Universal Deployer Contract.
pub fn get_contract_address(
    salt: Felt,
    class_hash: Felt,
    constructor_calldata: &[Felt],
    deployer_address: Felt,
) -> Felt {
    normalize_address(compute_hash_on_elements(&[
        CONTRACT_ADDRESS_PREFIX,
        deployer_address,
        salt,
        class_hash,
        compute_hash_on_elements(constructor_calldata),
    ]))
}

#[allow(clippy::too_many_arguments)]
pub async fn get_deployment_result(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    class_hash: Felt,
    signing_key: SigningKey,
    salt: Felt,
    chain_id: Felt,
    max_fee: Option<Felt>,
    wait_config: WaitForTx,
) -> Result<Felt, CreationError> {
    match account_type {
        AccountType::Oz => {
            deploy_oz_account(
                provider,
                class_hash,
                signing_key,
                salt,
                chain_id,
                max_fee,
                wait_config,
            )
            .await
        }
    }
}

async fn deploy_oz_account(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: Felt,
    signing_key: SigningKey,
    salt: Felt,
    chain_id: Felt,
    max_fee: Option<Felt>,
    wait_config: WaitForTx,
) -> Result<Felt, CreationError> {
    let factory = OpenZeppelinAccountFactory::new(
        class_hash,
        chain_id,
        LocalWallet::from_signing_key(signing_key),
        provider,
    )
    .await
    .unwrap();

    deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
}

#[allow(unused_variables)]
async fn deploy_account<T>(
    account_factory: T,
    provider: &JsonRpcClient<HttpTransport>,
    salt: Felt,
    max_fee: Option<Felt>,
    wait_config: WaitForTx,
    class_hash: Felt,
) -> Result<Felt, CreationError>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy_v1(salt);
    let deploy_max_fee = if let Some(max_fee) = max_fee {
        max_fee
    } else {
        match deployment.estimate_fee().await {
            Ok(max_fee) => Felt::from_dec_str(&max_fee.overall_fee.to_string()).unwrap(),
            Err(error) => return Err(CreationError::RpcError(error.to_string())),
        }
    };
    let result = deployment.max_fee(deploy_max_fee).send().await.unwrap();

    Ok(result.transaction_hash)
}
