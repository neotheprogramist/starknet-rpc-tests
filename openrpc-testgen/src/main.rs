use openrpc_testgen::{
    suite_openrpc::{SetupInput, TestSuiteOpenRpc},
    RunnableTrait,
};
use starknet_types_core::felt::Felt;
use std::{path::PathBuf, str::FromStr};
use url::Url;
pub mod args;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let suite_openrpc_input = SetupInput {
        urls: vec![Url::from_str("http://127.0.0.1:5050").unwrap()],

        paymaster_account_address: Felt::from_hex_unchecked(
            "0x64b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
        ),
        paymaster_private_key: Felt::from_hex_unchecked(
            "0x0000000000000000000000000000000071d7bb07b9a64f6f78ac4c816aff4da9",
        ),
        udc_address: Felt::from_hex_unchecked(
            "0x41A78E741E5AF2FEC34B695679BC6891742439F7AFB8484ECD7766661AD02BF",
        ),
        executable_account_sierra_path: PathBuf::from_str(
            "target/dev/contracts_ExecutableAccount.contract_class.json",
        )
        .unwrap(),
        executable_account_casm_path: PathBuf::from_str(
            "target/dev/contracts_ExecutableAccount.compiled_contract_class.json",
        )
        .unwrap(),
        account_class_hash: Felt::from_hex_unchecked(
            "0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f",
        ),
    };
    // let suite_openrpc_input = SetupInput {
    //     urls: vec![Url::from_str("http://0.0.0.0:9944").unwrap()],

    //     paymaster_account_address: Felt::from_hex_unchecked(
    //         "0x008a1719e7ca19f3d91e8ef50a48fc456575f645497a1d55f30e3781f786afe4",
    //     ),
    //     paymaster_private_key: Felt::from_hex_unchecked(
    //         "0x0514977443078cf1e0c36bc88b89ada9a46061a5cf728f40274caea21d76f174",
    //     ),
    //     udc_address: Felt::from_hex_unchecked(
    //         "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf",
    //     ),
    //     executable_account_sierra_path: PathBuf::from_str(
    //         "target/dev/contracts_ExecutableAccount.contract_class.json",
    //     )
    //     .unwrap(),
    //     executable_account_casm_path: PathBuf::from_str(
    //         "target/dev/contracts_ExecutableAccount.compiled_contract_class.json",
    //     )
    //     .unwrap(),
    //     account_class_hash: Felt::from_hex_unchecked(
    //         "0xe2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6",
    //     ),
    // };

    let _ = TestSuiteOpenRpc::run(&suite_openrpc_input).await;
}
