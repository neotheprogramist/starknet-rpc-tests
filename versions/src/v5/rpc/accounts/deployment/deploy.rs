use std::future::Pending;

use starknet_types_rpc::{BlockId, BlockTag, Felt, TxnHash};

use crate::v5::rpc::{
    accounts::{
        creation::{create::AccountType, structs::GenerateAccountResponse},
        errors::CreationError,
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        provider::Provider,
    },
    signers::key_pair::SigningKey,
};

use super::{
    helpers::{get_contract_address, get_deployment_result},
    structs::{Deploy, WaitForTx},
};

pub async fn deploy(
    provider: JsonRpcClient<HttpTransport>,
    deploy_args: Deploy,
    chain_id: Felt,
    wait_config: WaitForTx,
    account: &str,
    account_data: GenerateAccountResponse,
) -> Result<TxnHash, CreationError> {
    if account_data.deployed {
        tracing::info!("Account already deployed!");
        return Ok(Felt::ZERO);
    }

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
        Felt::ZERO
    } else {
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

    Ok(result)
}
