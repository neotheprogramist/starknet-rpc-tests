use clap::{Parser, ValueEnum};
use starknet_types_core::felt::Felt;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, disable_version_flag = true)]
pub struct Args {
    #[arg(long, short = 'u', env)]
    pub url: Url,

    #[arg(long, short = 'l', env)]
    pub l1_network_url: Url,

    #[arg(long, short = 's', env)]
    pub sierra_path: String,

    #[arg(long, short = 'c', env)]
    pub casm_path: String,

    #[arg(long, env)]
    pub sierra_path_2: String,

    #[arg(long, env)]
    pub casm_path_2: String,

    #[arg(long, short, env)]
    pub version: Version,

    #[arg(
        long,
        short = 'p',
        env,
        requires = "account_address",
        requires = "account_class_hash",
        requires = "erc20_strk_contract_address",
        requires = "erc20_eth_contract_address",
        requires = "amount_per_test"
    )]
    pub private_key: Option<Felt>,

    #[arg(
        long,
        short = 'a',
        env,
        requires = "private_key",
        requires = "account_class_hash",
        requires = "erc20_strk_contract_address",
        requires = "erc20_eth_contract_address",
        requires = "amount_per_test"
    )]
    pub account_address: Option<Felt>,

    #[arg(
        long,
        env,
        requires = "private_key",
        requires = "account_address",
        requires = "erc20_strk_contract_address",
        requires = "erc20_eth_contract_address",
        requires = "amount_per_test"
    )]
    pub account_class_hash: Option<Felt>,

    #[arg(
        long,
        short = 'r',
        env,
        requires = "private_key",
        requires = "account_address",
        requires = "account_class_hash",
        requires = "erc20_eth_contract_address",
        requires = "amount_per_test"
    )]
    pub erc20_strk_contract_address: Option<Felt>,

    #[arg(
        long,
        short = 'e',
        env,
        requires = "private_key",
        requires = "account_address",
        requires = "account_class_hash",
        requires = "erc20_strk_contract_address",
        requires = "amount_per_test"
    )]
    pub erc20_eth_contract_address: Option<Felt>,

    #[arg(
        long,
        short = 'm',
        env,
        requires = "private_key",
        requires = "account_address",
        requires = "account_class_hash",
        requires = "erc20_strk_contract_address",
        requires = "erc20_eth_contract_address"
    )]
    pub amount_per_test: Option<Felt>,
    // #[arg(long, env, default = false)]
    // pub starknet_devnet_api_tests: bool,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum Version {
    V5,
    V6,
    V7,
}
