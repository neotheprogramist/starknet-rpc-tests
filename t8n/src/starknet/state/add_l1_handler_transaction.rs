use blockifier::transaction::transactions::ExecutableTransaction;
use starknet_devnet_types::felt::TransactionHash;
use starknet_devnet_types::rpc::transactions::l1_handler_transaction::L1HandlerTransaction;
use starknet_devnet_types::rpc::transactions::{Transaction, TransactionWithHash};
use tracing::trace;

use super::DevnetResult;
use super::DumpEvent;
use super::Starknet;

pub fn add_l1_handler_transaction(
    starknet: &mut Starknet,
    transaction: L1HandlerTransaction,
) -> DevnetResult<TransactionHash> {
    let blockifier_transaction =
        transaction.create_blockifier_transaction(starknet.chain_id().to_felt())?;
    let transaction_hash = blockifier_transaction.tx_hash.0.into();
    trace!(
        "Executing L1 handler transaction [{:#064x}]",
        transaction_hash
    );

    // Fees are charges on L1 as `L1HandlerTransaction` is not executed by an
    // account, but directly by the sequencer.
    // https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/messaging-mechanism/#l1-l2-message-fees
    let charge_fee = false;
    let validate = true;

    let blockifier_execution_result = blockifier_transaction.execute(
        &mut starknet.state.state,
        &starknet.block_context,
        charge_fee,
        validate,
    );

    starknet.handle_transaction_result(
        TransactionWithHash::new(
            transaction_hash,
            Transaction::L1Handler(transaction.clone()),
        ),
        None,
        blockifier_execution_result,
    )?;
    starknet.handle_dump_event(DumpEvent::AddL1HandlerTransaction(transaction))?;

    Ok(transaction_hash)
}
