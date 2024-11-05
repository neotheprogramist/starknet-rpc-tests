// use std::path::PathBuf;

// use clap::{Parser, ValueEnum};
// use openrpc_testgen::structs::contract_path_pair::ContractPathPair;
// use starknet_types_core::felt::Felt;
// use url::Url;

// #[derive(Parser, Debug, Clone)]
// #[command(version, about, long_about = None, disable_version_flag = true)]
// pub struct CliArgs {
//     #[arg(long, short = 'u', env, help = "URL of the L2 node")]
//     pub url: Url,
//     #[arg(long, short = 'a', env, help = "Address of the paymaster")]
//     pub paymaster_account_address: Felt,
//     #[arg(long, short = 'k', env, help = "Private key of the paymaster")]
//     pub paymaster_private_key: Felt,
//     #[arg(long, short = 'u', env, help = "Address of the UDC contract")]
//     pub udc_address: Felt,
//     #[arg(long, short = 'k', env, help = "Executable account sierra path")]
//     pub executable_account_seirra_path: PathBuf,
//     pub executable_account_casm_path: PathBuf,
//     pub contracts_to_deploy_paths: Vec<ContractPathPair>,
// }
