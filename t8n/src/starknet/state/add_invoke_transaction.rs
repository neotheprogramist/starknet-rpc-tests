use blockifier::transaction::transactions::ExecutableTransaction;
use starknet_devnet_types::felt::TransactionHash;
use starknet_devnet_types::rpc::transactions::invoke_transaction_v1::InvokeTransactionV1;
use starknet_devnet_types::rpc::transactions::invoke_transaction_v3::InvokeTransactionV3;
use starknet_devnet_types::rpc::transactions::{
    BroadcastedInvokeTransaction, InvokeTransaction, Transaction, TransactionWithHash,
};

use super::dump::DumpEvent;
use super::Starknet;
use super::{DevnetResult, Error};

pub fn add_invoke_transaction(
    starknet: &mut Starknet,
    broadcasted_invoke_transaction: BroadcastedInvokeTransaction,
) -> DevnetResult<TransactionHash> {
    if broadcasted_invoke_transaction.is_max_fee_zero_value() {
        return Err(Error::MaxFeeZeroError {
            tx_type: broadcasted_invoke_transaction.to_string(),
        });
    }

    let blockifier_invoke_transaction = broadcasted_invoke_transaction
        .create_blockifier_invoke_transaction(&starknet.chain_id().to_felt())?;

    if blockifier_invoke_transaction.only_query {
        return Err(Error::UnsupportedAction {
            msg: "query-only transactions are not supported".to_string(),
        });
    }

    let transaction_hash = blockifier_invoke_transaction.tx_hash.0.into();

    let invoke_transaction = match broadcasted_invoke_transaction {
        BroadcastedInvokeTransaction::V1(ref v1) => {
            Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1::new(v1)))
        }
        BroadcastedInvokeTransaction::V3(ref v3) => {
            Transaction::Invoke(InvokeTransaction::V3(InvokeTransactionV3::new(v3)))
        }
    };

    let blockifier_execution_result =
        blockifier::transaction::account_transaction::AccountTransaction::Invoke(
            blockifier_invoke_transaction,
        )
        .execute(
            &mut starknet.state.state,
            &starknet.block_context,
            true,
            true,
        );

    let transaction = TransactionWithHash::new(transaction_hash, invoke_transaction);

    starknet.handle_transaction_result(transaction, None, blockifier_execution_result)?;
    starknet.handle_dump_event(DumpEvent::AddInvokeTransaction(
        broadcasted_invoke_transaction,
    ))?;

    Ok(transaction_hash)
}
