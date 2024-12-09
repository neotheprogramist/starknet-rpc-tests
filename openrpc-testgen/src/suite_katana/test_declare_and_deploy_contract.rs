use crate::{
    assert_eq_result, assert_matches_result, assert_result,
    utils::v7::{
        accounts::{
            account::{Account, ConnectedAccount},
            call::Call,
            deployment::helpers::get_contract_address,
        },
        endpoints::{
            declare_contract::prepare_contract_declaration_params,
            errors::OpenRpcTestGenError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag, DeclareTxnReceipt, TxnReceipt};

use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let path: PathBuf =
            PathBuf::from("openrpc-testgen/src/suite_katana/test_data/cairo1_contract.json");

        let (flattened_sierra_class, compiled_class_hash) =
            prepare_contract_declaration_params(&path)?;

        let account = test_input.random_paymaster_account.random_accounts()?;

        let provider = account.provider().clone();

        let declare_res = test_input
            .random_paymaster_account
            .declare_v2(
                Arc::new(flattened_sierra_class.clone()),
                compiled_class_hash,
            )
            .send()
            .await?;

        let (transaction_hash, class_hash) = (declare_res.transaction_hash, declare_res.class_hash);
        wait_for_sent_transaction(transaction_hash, &account).await?;

        // check that the tx is executed successfully and return the correct receipt
        let receipt = provider.get_transaction_receipt(transaction_hash).await?;
        assert_matches_result!(receipt, TxnReceipt::Declare(DeclareTxnReceipt { .. }));

        // check that the class is actually declared
        let get_class_ok = provider
            .clone()
            .get_class(BlockId::Tag(BlockTag::Pending), class_hash)
            .await
            .is_ok();

        assert_result!(get_class_ok);

        let ctor_args = vec![Felt::ONE, Felt::TWO];
        let calldata = [
            vec![
                declare_res.class_hash,      // class hash
                Felt::ZERO,                  // salt
                Felt::ZERO,                  // unique
                Felt::from(ctor_args.len()), // constructor calldata len
            ],
            ctor_args.clone(),
        ]
        .concat();

        // pre-compute the contract address of the would-be deployed contract
        let address =
            get_contract_address(Felt::ZERO, declare_res.class_hash, &ctor_args, Felt::ZERO);

        let res = test_input
            .random_paymaster_account
            .execute_v1(vec![Call {
                calldata,
                to: Felt::from_hex_unchecked(
                    "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf", //DEFAULT_UDC_ADDRESS
                ),
                selector: get_selector_from_name("deployContract")?,
            }])
            .send()
            .await?;

        wait_for_sent_transaction(
            res.transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        // make sure the contract is deployed
        let res = provider
            .get_class_hash_at(BlockId::Tag(BlockTag::Pending), address)
            .await?;

        assert_eq_result!(res, class_hash);

        Ok(Self {})
    }
}
