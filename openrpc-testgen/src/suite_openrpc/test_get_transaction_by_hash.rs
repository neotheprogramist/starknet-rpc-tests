use crate::utils::v7::accounts::account::Account;
use crate::utils::v7::accounts::call::Call;
use crate::utils::v7::endpoints::utils::wait_for_sent_transaction;
use crate::RandomizableAccountsTrait;
use crate::{
    utils::v7::{
        accounts::{
            account::ConnectedAccount,
            creation::create::{create_account, AccountType},
        },
        endpoints::{errors::RpcError, utils::get_selector_from_name},
        providers::provider::Provider,
        signers::key_pair::SigningKey,
    },
    RunnableTrait,
};
use colored::Colorize;
use starknet_types_core::felt::Felt;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteOpenRpc;
    type Output = ();

    async fn run(test_input: &Self::Input) -> Result<Self::Output, RpcError> {
        let created_account_data = create_account(
            &test_input.random_paymaster_account.provider(),
            AccountType::Oz,
            Option::None,
            Some(Felt::from_hex(
                "0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
            )?),
        )
        .await?;

        let deploy_account_account_call = Call {
            to: Felt::from_hex(
                "0x41A78E741E5AF2FEC34B695679BC6891742439F7AFB8484ECD7766661AD02BF",
            )?,
            selector: get_selector_from_name("deployContract")?,
            calldata: vec![
                Felt::from_hex(
                    "0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
                )?,
                created_account_data.salt,
                Felt::ZERO,
                Felt::ONE,
                SigningKey::verifying_key(&created_account_data.signing_key).scalar(),
            ],
        };

        let deploy_account_result = test_input
            .random_paymaster_account
            .execute_v3(vec![deploy_account_account_call])
            .send()
            .await?;

        wait_for_sent_transaction(
            deploy_account_result.transaction_hash,
            &test_input.random_paymaster_account.random_accounts()?,
        )
        .await?;

        let txn = test_input
            .random_paymaster_account
            .provider()
            .get_transaction_by_hash(deploy_account_result.transaction_hash)
            .await;

        match txn {
            Ok(_) => {
                info!(
                    "{} {}",
                    "✓ Rpc get_storage_at COMPATIBLE".green(),
                    "✓".green()
                );
            }
            Err(e) => {
                error!(
                    "{} {} {}",
                    "✗ Rpc get_storage_at INCOMPATIBLE:".red(),
                    e.to_string().red(),
                    "✗".red()
                );
            }
        }

        Ok(())
    }
}
