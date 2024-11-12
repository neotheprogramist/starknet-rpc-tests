use std::path::PathBuf;

use clap::Parser;
use starknet_types_core::felt::Felt;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, disable_version_flag = true)]
pub struct Args {
    #[arg(
        long,
        env,
        help = "Space-separated URLs of the L2 nodes (e.g. 'http://127.0.0.1:5050 http://127.0.0.1:5050')",
        value_delimiter = ' '
    )]
    pub urls: Vec<Url>,

    #[arg(long, env, help = "Address of an account that would pay for fees")]
    pub paymaster_account_address: Felt,

    #[arg(long, env, help = "Private Key of an account that would pay for fees")]
    pub paymaster_private_key: Felt,

    #[arg(long, env, help = "Universal Deployer Contract address")]
    pub udc_address: Felt,

    #[arg(
        long,
        env,
        help = "Path to sierra of Executable Account contract (necessary to delegate fee payment to paymaster)"
    )]
    pub executable_account_sierra_path: PathBuf,

    #[arg(
        long,
        env,
        help = "Path to casm of Executable Account contract (necessary to delegate fee payment to paymaster)"
    )]
    pub executable_account_casm_path: PathBuf,

    #[arg(long, env, help = "Class hash of account contract")]
    pub account_class_hash: Felt,
}
