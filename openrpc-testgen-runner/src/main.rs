use args::{Args, Suite};
use clap::Parser;
#[allow(unused_imports)]
use openrpc_testgen::{
    suite_katana::{SetupInput as SetupInputKatana, TestSuiteKatana},
    suite_katana_no_account_validation::{
        SetupInput as SetupInputKatanaNoAccountValidation, TestSuiteKatanaNoAccountValidation,
    },
    suite_katana_no_fee::{SetupInput as SetupInputKatanaNoFee, TestSuiteKatanaNoFee},
    suite_katana_no_mining::{SetupInput as SetupInputKatanaNoMining, TestSuiteKatanaNoMining},
    suite_openrpc::{SetupInput, TestSuiteOpenRpc},
    RunnableTrait,
};
use tracing::error;
pub mod args;

#[tokio::main]
#[allow(unused_variables)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let args = Args::parse();

    for suite in args.suite {
        match suite {
            Suite::OpenRpc => {
                #[cfg(feature = "openrpc")]
                {
                    let suite_openrpc_input = SetupInput {
                        urls: args.urls.clone(),
                        paymaster_account_address: args.paymaster_account_address.clone(),
                        paymaster_private_key: args.paymaster_private_key.clone(),
                        udc_address: args.udc_address.clone(),
                        account_class_hash: args.account_class_hash.clone(),
                    };
                    if let Err(e) = TestSuiteOpenRpc::run(&suite_openrpc_input).await {
                        error!("Error while running TestSuiteOpenRpc: {}", e);
                    }
                }
                #[cfg(not(feature = "openrpc"))]
                {
                    error!("Feature 'openrpc' not enabled during compilation phase.");
                }
            }
            Suite::Katana => {
                #[cfg(feature = "katana")]
                {
                    let suite_katana_input = SetupInputKatana {
                        urls: args.urls.clone(),
                        paymaster_account_address: args.paymaster_account_address.clone(),
                        paymaster_private_key: args.paymaster_private_key.clone(),
                        udc_address: args.udc_address.clone(),
                        account_class_hash: args.account_class_hash.clone(),
                    };
                    if let Err(e) = TestSuiteKatana::run(&suite_katana_input).await {
                        error!("Error while running TestSuiteKatana: {}", e);
                    }
                }
                #[cfg(not(feature = "katana"))]
                {
                    error!("Feature 'katana' not enabled during compilation phase.");
                }
            }
            Suite::KatanaNoMining => {
                #[cfg(feature = "katana_no_mining")]
                {
                    let suite_katana_no_mining_input = SetupInputKatanaNoMining {
                        urls: args.urls.clone(),
                        paymaster_account_address: args.paymaster_account_address.clone(),
                        paymaster_private_key: args.paymaster_private_key.clone(),
                        udc_address: args.udc_address.clone(),
                        account_class_hash: args.account_class_hash.clone(),
                    };
                    if let Err(e) =
                        TestSuiteKatanaNoMining::run(&suite_katana_no_mining_input).await
                    {
                        error!("Error while running TestSuiteKatanaNoMining: {}", e);
                    }
                }
                #[cfg(not(feature = "katana_no_mining"))]
                {
                    error!("Feature 'katana_no_mining' not enabled during compilation phase.");
                }
            }
            Suite::KatanaNoFee => {
                #[cfg(feature = "katana_no_fee")]
                {
                    let suite_katana_no_fee_input = SetupInputKatanaNoFee {
                        urls: args.urls.clone(),
                        paymaster_account_address: args.paymaster_account_address.clone(),
                        paymaster_private_key: args.paymaster_private_key.clone(),
                        udc_address: args.udc_address.clone(),
                        account_class_hash: args.account_class_hash.clone(),
                    };
                    if let Err(e) = TestSuiteKatanaNoFee::run(&suite_katana_no_fee_input).await {
                        error!("Error while running TestSuiteKatanaNoFee: {}", e);
                    }
                }
                #[cfg(not(feature = "katana_no_fee"))]
                {
                    error!("Feature 'katana_no_fee' not enabled during compilation phase.");
                }
            }
            Suite::KatanaNoAccountValidation => {
                #[cfg(feature = "katana_no_account_validation")]
                {
                    let suite_katana_no_account_validation_input =
                        SetupInputKatanaNoAccountValidation {
                            urls: args.urls.clone(),
                            paymaster_account_address: args.paymaster_account_address.clone(),
                            paymaster_private_key: args.paymaster_private_key.clone(),
                            udc_address: args.udc_address.clone(),
                            account_class_hash: args.account_class_hash.clone(),
                        };
                    if let Err(e) = TestSuiteKatanaNoAccountValidation::run(
                        &suite_katana_no_account_validation_input,
                    )
                    .await
                    {
                        error!(
                            "Error while running TestSuiteKatanaNoAccountValidation: {}",
                            e
                        );
                    }
                }
                #[cfg(not(feature = "katana_no_account_validation"))]
                {
                    error!("Feature 'katana_no_account_validation' not enabled during compilation phase.");
                }
            }
        }
    }
}
