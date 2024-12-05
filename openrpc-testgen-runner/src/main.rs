use args::Args;
use clap::Parser;
#[allow(unused_imports)]
use openrpc_testgen::{
    suite_katana::{SetupInput as SetupInputKatana, TestSuiteKatana},
    suite_katana_no_mining::{SetupInput as SetupInputKatanaNoMining, TestSuiteKatanaNoMining},
    suite_openrpc::{SetupInput, TestSuiteOpenRpc},
    RunnableTrait,
};
pub mod args;

#[tokio::main]
#[allow(unused_variables)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let args = Args::parse();

    #[cfg(feature = "openrpc")]
    {
        let suite_openrpc_input = SetupInput {
            urls: args.urls.clone(),
            paymaster_account_address: args.paymaster_account_address,
            paymaster_private_key: args.paymaster_private_key,
            udc_address: args.udc_address,
            account_class_hash: args.account_class_hash,
        };

        let _ = TestSuiteOpenRpc::run(&suite_openrpc_input).await;
    }

    #[cfg(feature = "katana")]
    {
        let suite_katana_input = SetupInputKatana {
            urls: args.urls.clone(),
            paymaster_account_address: args.paymaster_account_address,
            paymaster_private_key: args.paymaster_private_key,
            udc_address: args.udc_address,
            account_class_hash: args.account_class_hash,
        };

        let _ = TestSuiteKatana::run(&suite_katana_input).await;
    }
    #[cfg(feature = "katana_no_mining")]
    {
        let suite_katana_input = SetupInputKatanaNoMining {
            urls: args.urls.clone(),
            paymaster_account_address: args.paymaster_account_address,
            paymaster_private_key: args.paymaster_private_key,
            udc_address: args.udc_address,
            executable_account_sierra_path: args.executable_account_sierra_path.clone(),
            executable_account_casm_path: args.executable_account_casm_path.clone(),
            account_class_hash: args.account_class_hash,
        };

        let _ = TestSuiteKatanaNoMining::run(&suite_katana_input).await;
    }
}
