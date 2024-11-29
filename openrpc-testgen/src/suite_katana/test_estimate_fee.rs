use crate::{
    assert_result,
    utils::v7::{
        accounts::account::{Account, ConnectedAccount},
        contract::factory::ContractFactory,
        endpoints::{
            declare_contract::get_compiled_contract, errors::OpenRpcTestGenError,
            utils::wait_for_sent_transaction,
        },
        providers::provider::Provider,
    },
    RandomizableAccountsTrait, RunnableTrait,
};

use rand::{rngs::StdRng, RngCore, SeedableRng};
use starknet_types_core::felt::Felt;
use starknet_types_rpc::{BlockId, BlockTag};

use std::{path::PathBuf, str::FromStr, sync::Arc};

#[derive(Clone, Debug)]
pub struct TestCase {}

impl RunnableTrait for TestCase {
    type Input = super::TestSuiteKatana;
    async fn run(test_input: &Self::Input) -> Result<Self, OpenRpcTestGenError> {
        let (flattened_sierra_class, compiled_class_hash) = get_compiled_contract(
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_3_HelloStarknet.contract_class.json")?,
            PathBuf::from_str("target/dev/contracts_contracts_sample_contract_3_HelloStarknet.compiled_contract_class.json")?,
        )
        .await?;

        let account = test_input.random_paymaster_account.random_accounts()?;
        let provider = account.provider().clone();

        // send a valid transaction first to increment the nonce (so that we can test nonce < current
        // nonce later)
        let declare_res = account
            .declare_v2(
                Arc::new(flattened_sierra_class.clone()),
                compiled_class_hash,
            )
            .send()
            .await?;

        let (transaction_hash, _class_hash) =
            (declare_res.transaction_hash, declare_res.class_hash);

        wait_for_sent_transaction(transaction_hash, &account).await?;

        let factory = ContractFactory::new(
            declare_res.class_hash,
            test_input.random_paymaster_account.random_accounts()?,
        );

        let mut salt_buffer = [0u8; 32];
        let mut rng = StdRng::from_entropy();
        rng.fill_bytes(&mut salt_buffer[1..]);

        // estimate fee with current nonce (the expected nonce)
        let nonce = provider
            .get_nonce(BlockId::Tag(BlockTag::Pending), account.address())
            .await?;

        let estimate_result = factory
            .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .nonce(nonce)
            .estimate_fee()
            .await;

        let estimate_is_ok = estimate_result.is_ok();
        assert_result!(
            estimate_is_ok,
            "estimate should succeed with nonce == current nonce"
        );

        // estimate fee with arbitrary nonce < current nonce
        //
        // here we're essentially estimating a transaction with a nonce that has already been
        // used, so it should fail.
        let nonce = nonce - 1;
        let estimate_result = factory
            .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .nonce(nonce)
            .estimate_fee()
            .await;

        let estimate_is_err = estimate_result.is_err();
        assert_result!(
            estimate_is_err,
            "estimate should fail with nonce < current nonce"
        );

        let nonce = Felt::from_hex_unchecked("0x1337");
        let estimate_result = factory
            .deploy_v3(vec![], Felt::from_bytes_be(&salt_buffer), true)
            .nonce(nonce)
            .estimate_fee()
            .await;
        let estimate_is_ok = estimate_result.is_ok(); // TODO: On starknet-devnet error
        assert_result!(
            estimate_is_ok,
            "estimate should succeed with nonce >= current nonce"
        );

        Ok(Self {})
    }
}
