use args::Args;
use clap::Parser;
use openrpc_testgen::{
    suite_openrpc::{SetupInput, TestSuiteOpenRpc},
    RunnableTrait,
};
pub mod args;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let args = Args::parse();

    let suite_openrpc_input = SetupInput {
        urls: args.urls,
        paymaster_account_address: args.paymaster_account_address,
        paymaster_private_key: args.paymaster_private_key,
        udc_address: args.udc_address,
        executable_account_sierra_path: args.executable_account_sierra_path,
        executable_account_casm_path: args.executable_account_casm_path,
        account_class_hash: args.account_class_hash,
    };

    let _ = TestSuiteOpenRpc::run(&suite_openrpc_input).await;
}
