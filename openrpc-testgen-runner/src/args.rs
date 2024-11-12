use std::path::PathBuf;

use clap::Parser;
use starknet_types_core::felt::Felt;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, disable_version_flag = true)]
pub struct Args {
    #[arg(long, env, help = "URL of the L2 node", value_delimiter = ' ')]
    pub urls: Vec<Url>,

    #[arg(long, env)]
    pub paymaster_account_address: Felt,

    #[arg(long, env)]
    pub paymaster_private_key: Felt,

    #[arg(long, env)]
    pub udc_address: Felt,

    #[arg(long, env)]
    pub executable_account_sierra_path: PathBuf,

    #[arg(long, env)]
    pub executable_account_casm_path: PathBuf,

    #[arg(long, env)]
    pub account_class_hash: Felt,
}
