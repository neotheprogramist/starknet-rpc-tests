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
    };

    let _ = TestSuiteOpenRpc::run(&suite_openrpc_input).await;
}
