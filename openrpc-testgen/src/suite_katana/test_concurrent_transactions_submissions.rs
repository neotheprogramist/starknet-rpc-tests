use std::sync::Arc;

use crate::{
    assert_eq_result,
    utils::v7::{
        accounts::{
            account::{Account, ConnectedAccount},
            call::Call,
        },
        endpoints::{
            errors::OpenRpcTestGenError,
            utils::{get_selector_from_name, wait_for_sent_transaction},
        },
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, RunnableTrait,
};
use indexmap::IndexSet;

use starknet_types_core::felt::Felt;
use starknet_types_rpc::{TxnExecutionStatus, TxnStatus};
use tokio::sync::Mutex;

pub const DEFAULT_PREFUNDED_ACCOUNT_BALANCE: u128 = 10 * u128::pow(10, 21);

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let account = test_input.random_paymaster_account.random_accounts()?;

        let provider = account.provider().clone();

        let initial_nonce = account.get_nonce().await?;

        const N: usize = 100;
        let nonce = Arc::new(Mutex::new(initial_nonce));
        let txs = Arc::new(Mutex::new(IndexSet::with_capacity(N)));

        let mut handles = Vec::with_capacity(N);

        for _ in 0..N {
            let txs = txs.clone();
            let nonce = nonce.clone();
            let account = account.clone();
            let deployed_contract_address = test_input.deployed_contract_address;

            let handle = tokio::spawn(async move {
                let mut nonce = nonce.lock().await;
                let res = account
                    .execute_v1(vec![Call {
                        to: deployed_contract_address,
                        selector: get_selector_from_name("increase_balance").unwrap(),
                        calldata: vec![Felt::from_hex("0x50").unwrap()],
                    }])
                    .nonce(*nonce)
                    .send()
                    .await
                    .unwrap();
                txs.lock().await.insert(res.transaction_hash);
                *nonce += Felt::ONE;
            });

            handles.push(handle);
        }

        // wait for all txs to be submitted
        for handle in handles {
            handle.await?;
        }

        // Wait only for the last transaction to be accepted
        let txs = txs.lock().await;
        let last_tx = txs.last().unwrap();
        wait_for_sent_transaction(*last_tx, &account).await?;

        // we should've submitted ITERATION transactions
        assert_eq_result!(txs.len(), N);

        // check the status of each txs
        for hash in txs.iter() {
            let txn_finality_and_execution_status = provider.get_transaction_status(*hash).await?;
            assert_eq_result!(
                txn_finality_and_execution_status.execution_status,
                Some(TxnExecutionStatus::Succeeded)
            );
            assert_eq_result!(
                txn_finality_and_execution_status.finality_status,
                TxnStatus::AcceptedOnL2
            );
        }

        let nonce = account.get_nonce().await?;

        assert_eq_result!(
            nonce,
            initial_nonce + Felt::from(N),
            "Nonce should be incremented by {N} time"
        );

        Ok(Self {})
    }
}
