use crate::{
    assert_result,
    utils::v7::{
        accounts::{
            account::{Account, ConnectedAccount},
            call::Call,
        },
        contract::factory::ContractFactory,
        endpoints::{
            declare_contract::get_compiled_contract,
            endpoints_functions::OutsideExecution,
            errors::{CallError, RpcError},
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use cainome_cairo_serde::CairoSerde;
use colored::Colorize;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet::core::crypto::ecdsa_sign;
use starknet_types_core::{
    felt::Felt,
    hash::{Poseidon, StarkHash},
};
use starknet_types_rpc::{BlockId, BlockTag, FunctionCall, TxnReceipt};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;

    async fn run(test_input: &Self::Input) -> Result<Self, RpcError> {
        let (erc_20_flattened_sierra_class, erc_20_compiled_class_hash) = get_compiled_contract(
            PathBuf::from_str("target/dev/contracts_TestToken.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_TestToken.compiled_contract_class.json")?,
        )
        .await?;

        let declaration_hash = test_input
            .random_paymaster_account
            .declare_v3(erc_20_flattened_sierra_class, erc_20_compiled_class_hash)
            .send()
            .await?;

        let factory = ContractFactory::new(
            declaration_hash.class_hash,
            test_input.random_paymaster_account.random_accounts()?,
        );
        let mut salt_buffer = [0u8; 32];
        let mut rng = StdRng::from_entropy();
        rng.fill_bytes(&mut salt_buffer[1..]);

        let deployment_result = factory
            .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .send()
            .await?;

        wait_for_sent_transaction(
            deployment_result.transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        let deployment_receipt_erc20 = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_receipt(deployment_result.transaction_hash)
            .await?;

        let contract_address_erc20 = match deployment_receipt_erc20 {
            TxnReceipt::Deploy(receipt) => receipt.contract_address,
            TxnReceipt::Invoke(receipt) => {
                if let Some(contract_address) = receipt
                    .common_receipt_properties
                    .events
                    .first()
                    .and_then(|event| event.data.first())
                {
                    *contract_address
                } else {
                    return Err(RpcError::CallError(CallError::UnexpectedReceiptType));
                }
            }
            _ => {
                return Err(RpcError::CallError(CallError::UnexpectedReceiptType));
            }
        };

        let erc20_mint_call = Call {
            to: contract_address_erc20,
            selector: get_selector_from_name("mint")?,
            calldata: vec![
                test_input
                    .random_executable_account
                    .random_accounts()?
                    .address(),
                Felt::from_hex("0x123")?,
                Felt::ZERO,
            ],
        };

        test_input
            .random_paymaster_account
            .execute_v3(vec![erc20_mint_call])
            .send()
            .await?;

        let account_erc20_receiver_address =
            Felt::from_hex("0x78662e7352d062084b0010068b99288486c2d8b914f6e2a55ce945f8792c8b1")?;
        let amount_to_transfer = vec![Felt::from_hex("0x100")?, Felt::ZERO];

        let erc20_transfer_call = Call {
            to: contract_address_erc20,
            selector: get_selector_from_name("transfer")?,
            calldata: vec![
                account_erc20_receiver_address,
                amount_to_transfer[0],
                amount_to_transfer[1],
            ],
        };

        let outside_execution = OutsideExecution {
            caller: test_input
                .random_paymaster_account
                .random_accounts()?
                .address(),
            nonce: Felt::ZERO,
            calls: vec![erc20_transfer_call],
        };

        let outside_execution_cairo_serialized =
            &OutsideExecution::cairo_serialize(&outside_execution);

        let hash = Poseidon::hash_array(outside_execution_cairo_serialized);

        let starknet::core::crypto::ExtendedSignature { r, s, v: _ } = ecdsa_sign(
            &Felt::from_hex("0x0000000000000000000000000000000071d7bb07b9a64f6f78ac4c816aff4da9")?, //paymaster pk
            &hash,
        )
        .unwrap();

        let mut calldata_to_executable_account_call = outside_execution_cairo_serialized.clone();
        calldata_to_executable_account_call.push(Felt::from_dec_str("2")?);
        calldata_to_executable_account_call.push(r);
        calldata_to_executable_account_call.push(s);

        let call_to_executable_account = Call {
            to: test_input
                .random_executable_account
                .random_accounts()?
                .address(),
            selector: get_selector_from_name("execute_from_outside")?,
            calldata: calldata_to_executable_account_call,
        };

        let balance_before_transfer = test_input
            .random_executable_account
            .provider()
            .call(
                FunctionCall {
                    calldata: vec![test_input
                        .random_executable_account
                        .random_accounts()?
                        .address()],
                    contract_address: contract_address_erc20,
                    entry_point_selector: get_selector_from_name("balance_of")?,
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await?;

        let gas_balance_before = test_input
            .random_paymaster_account
            .provider()
            .call(
                FunctionCall {
                    calldata: vec![test_input
                        .random_paymaster_account
                        .random_accounts()?
                        .address()],
                    contract_address: Felt::from_hex(
                        "0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7",
                    )?,
                    entry_point_selector: get_selector_from_name("balance_of")?,
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await?;

        test_input
            .random_paymaster_account
            .execute_v1(vec![call_to_executable_account])
            .send()
            .await?;

        let balance_after_transfer = test_input
            .random_executable_account
            .provider()
            .call(
                FunctionCall {
                    calldata: vec![test_input
                        .random_executable_account
                        .random_accounts()?
                        .address()],
                    contract_address: contract_address_erc20,
                    entry_point_selector: get_selector_from_name("balance_of")?,
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await?;

        let gas_balance_after = test_input
            .random_paymaster_account
            .provider()
            .call(
                FunctionCall {
                    calldata: vec![test_input
                        .random_paymaster_account
                        .random_accounts()?
                        .address()],
                    contract_address: Felt::from_hex(
                        "0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7",
                    )?,
                    entry_point_selector: get_selector_from_name("balance_of")?,
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await?;

        let balance_after_txn = test_input
            .random_paymaster_account
            .provider()
            .call(
                FunctionCall {
                    calldata: vec![account_erc20_receiver_address],
                    contract_address: contract_address_erc20,
                    entry_point_selector: get_selector_from_name("balance_of")?,
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await?;

        assert_result!(
            balance_after_txn == amount_to_transfer,
            "Balances do not match"
        )?;
        assert_result!(
            balance_before_transfer > balance_after_transfer,
            "Token balance on executable account did not decrease as expected."
        )?;

        assert_result!(
            gas_balance_before > gas_balance_after,
            "Gas balance on paymaster account did not decrease after transaction."
        )?;

        info!(
            "{} {}",
            "\n✓ Rpc test_erc20_transfer COMPATIBLE".green(),
            "✓".green()
        );

        Ok(Self {})
    }
}
