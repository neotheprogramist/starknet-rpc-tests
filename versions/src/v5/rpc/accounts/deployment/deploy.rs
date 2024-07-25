use starknet_types_rpc::{BlockId, BlockTag, Felt, TxnHash};
use tracing::info;

use crate::v5::rpc::{
    accounts::{
        creation::{create::AccountType, structs::GenerateAccountResponse},
        errors::CreationError,
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        provider::Provider,
    },
};

use super::{
    helpers::{get_contract_address, get_deployment_result},
    structs::WaitForTx,
};

pub async fn deploy_account(
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    wait_config: WaitForTx,
    account_data: GenerateAccountResponse,
) -> Result<TxnHash, CreationError> {
    if account_data.deployed {
        tracing::info!("Account already deployed!");
        return Ok(Felt::ZERO);
    }
    info!("==============DEPLOY START..==============");
    let public_key = account_data.signing_key.verifying_key();

    let address = match account_data.account_type {
        AccountType::Oz => get_contract_address(
            account_data.salt,
            account_data.class_hash,
            &[public_key.scalar(), Felt::ZERO],
            Felt::ZERO,
        ),
    };

    let result = if provider
        .get_class_hash_at(BlockId::Tag(BlockTag::Pending), address)
        .await
        .is_ok()
    {
        info!("CLASS HASH AT IS OK");
        Felt::ZERO
    } else {
        info!("Getting deployment result");
        get_deployment_result(
            &provider,
            account_data.account_type,
            account_data.class_hash,
            account_data.signing_key,
            account_data.salt,
            chain_id,
            Some(account_data.max_fee),
            wait_config,
        )
        .await?
    };
    info!("DEPLOY END..");
    Ok(result)
}
