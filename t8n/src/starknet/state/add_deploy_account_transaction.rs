use blockifier::transaction::transactions::ExecutableTransaction;
use starknet_devnet_types::contract_address::ContractAddress;
use starknet_devnet_types::felt::TransactionHash;
use starknet_devnet_types::rpc::transactions::deploy_account_transaction_v1::DeployAccountTransactionV1;
use starknet_devnet_types::rpc::transactions::deploy_account_transaction_v3::DeployAccountTransactionV3;
use starknet_devnet_types::rpc::transactions::{
    BroadcastedDeployAccountTransaction, DeployAccountTransaction, Transaction, TransactionWithHash,
};

use super::dump::DumpEvent;
use super::errors::{DevnetResult, Error, StateError};
use super::starknet_state::CustomStateReader;
use super::Starknet;

pub fn add_deploy_account_transaction(
    starknet: &mut Starknet,
    broadcasted_deploy_account_transaction: BroadcastedDeployAccountTransaction,
) -> DevnetResult<(TransactionHash, ContractAddress)> {
    if broadcasted_deploy_account_transaction.is_max_fee_zero_value() {
        return Err(Error::MaxFeeZeroError {
            tx_type: broadcasted_deploy_account_transaction.to_string(),
        });
    }
    let blockifier_deploy_account_transaction = broadcasted_deploy_account_transaction
        .create_blockifier_deploy_account(&starknet.chain_id().to_felt())?;

    if blockifier_deploy_account_transaction.only_query {
        return Err(Error::UnsupportedAction {
            msg: "query-only transactions are not supported".to_string(),
        });
    }

    let address = blockifier_deploy_account_transaction
        .contract_address
        .into();

    let (class_hash, deploy_account_transaction) = match broadcasted_deploy_account_transaction {
        BroadcastedDeployAccountTransaction::V1(ref v1) => {
            let deploy_account_transaction =
                Transaction::DeployAccount(DeployAccountTransaction::V1(Box::new(
                    DeployAccountTransactionV1::new(v1, address),
                )));

            (v1.class_hash, deploy_account_transaction)
        }
        BroadcastedDeployAccountTransaction::V3(ref v3) => {
            let deploy_account_transaction =
                Transaction::DeployAccount(DeployAccountTransaction::V3(Box::new(
                    DeployAccountTransactionV3::new(v3, address),
                )));

            (v3.class_hash, deploy_account_transaction)
        }
    };

    if !starknet.state.is_contract_declared(class_hash) {
        return Err(Error::StateError(StateError::NoneClassHash(class_hash)));
    }
    let transaction_hash = blockifier_deploy_account_transaction.tx_hash.0.into();
    let transaction = TransactionWithHash::new(transaction_hash, deploy_account_transaction);

    let blockifier_execution_result =
        blockifier::transaction::account_transaction::AccountTransaction::DeployAccount(
            blockifier_deploy_account_transaction,
        )
        .execute(
            &mut starknet.state.state,
            &starknet.block_context,
            true,
            true,
        );

    starknet.handle_transaction_result(transaction, None, blockifier_execution_result)?;
    starknet.handle_dump_event(DumpEvent::AddDeployAccountTransaction(
        broadcasted_deploy_account_transaction,
    ))?;

    Ok((transaction_hash, address))
}
