use blockifier::transaction::transactions::ExecutableTransaction;
use starknet_devnet_types::felt::{ClassHash, TransactionHash};
use starknet_devnet_types::rpc::transactions::declare_transaction_v0v1::DeclareTransactionV0V1;
use starknet_devnet_types::rpc::transactions::declare_transaction_v2::DeclareTransactionV2;
use starknet_devnet_types::rpc::transactions::declare_transaction_v3::DeclareTransactionV3;
use starknet_devnet_types::rpc::transactions::{
    BroadcastedDeclareTransaction, DeclareTransaction, Transaction, TransactionWithHash,
};

use super::dump::DumpEvent;
use super::errors::{DevnetResult, Error};
use super::Starknet;

pub fn add_declare_transaction(
    starknet: &mut Starknet,
    broadcasted_declare_transaction: BroadcastedDeclareTransaction,
) -> DevnetResult<(TransactionHash, ClassHash)> {
    if broadcasted_declare_transaction.is_max_fee_zero_value() {
        return Err(Error::MaxFeeZeroError {
            tx_type: broadcasted_declare_transaction.to_string(),
        });
    }

    let blockifier_declare_transaction = broadcasted_declare_transaction
        .create_blockifier_declare(&starknet.chain_id().to_felt())?;

    if blockifier_declare_transaction.only_query() {
        return Err(Error::UnsupportedAction {
            msg: "query-only transactions are not supported".to_string(),
        });
    }

    let transaction_hash = blockifier_declare_transaction.tx_hash().0.into();
    let class_hash = blockifier_declare_transaction.class_hash().0.into();

    let (declare_transaction, contract_class) = match broadcasted_declare_transaction {
        BroadcastedDeclareTransaction::V1(ref v1) => {
            let declare_transaction = Transaction::Declare(DeclareTransaction::V1(
                DeclareTransactionV0V1::new(v1, class_hash),
            ));

            (declare_transaction, v1.contract_class.clone().into())
        }
        BroadcastedDeclareTransaction::V2(ref v2) => {
            let declare_transaction = Transaction::Declare(DeclareTransaction::V2(
                DeclareTransactionV2::new(v2, class_hash),
            ));

            (declare_transaction, v2.contract_class.clone().into())
        }
        BroadcastedDeclareTransaction::V3(ref v3) => {
            let declare_transaction = Transaction::Declare(DeclareTransaction::V3(
                DeclareTransactionV3::new(v3, class_hash),
            ));

            (declare_transaction, v3.contract_class.clone().into())
        }
    };

    let transaction = TransactionWithHash::new(transaction_hash, declare_transaction);
    let blockifier_execution_result =
        blockifier::transaction::account_transaction::AccountTransaction::Declare(
            blockifier_declare_transaction,
        )
        .execute(
            &mut starknet.state.state,
            &starknet.block_context,
            true,
            true,
        );

    starknet.handle_transaction_result(
        transaction,
        Some(contract_class),
        blockifier_execution_result,
    )?;

    starknet.handle_dump_event(DumpEvent::AddDeclareTransaction(
        broadcasted_declare_transaction,
    ))?;

    Ok((transaction_hash, class_hash))
}
