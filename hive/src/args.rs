use clap::Parser;
use starknet_types_core::felt::Felt;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, disable_version_flag = true)]
pub struct Args {
    #[arg(
        long,
        env,
        num_args = 1..,
        help = "A list of URLs for the service. A random URL from this list will be selected to send each test case."
    )]
    pub url_list: Vec<Url>,

    #[arg(long, short = 'l', env, help = "L1 network URL")]
    pub l1_network_url: Url,

    #[arg(long, short = 's', env, help = "Path to Sierra file")]
    pub sierra_path: String,

    #[arg(long, short = 'c', env, help = "Path to CASM file")]
    pub casm_path: String,

    #[arg(long, env, help = "Second Sierra path")]
    pub sierra_path_2: String,

    #[arg(long, env, help = "Second CASM path")]
    pub casm_path_2: String,

    #[arg(long, env, help = "Third Sierra path")]
    pub sierra_path_3: String,

    #[arg(long, env, help = "Third CASM path")]
    pub casm_path_3: String,

    #[arg(
        long,
        short = 'p',
        env,
        requires = "account_address",
        requires = "account_class_hash",
        requires = "erc20_strk_contract_address",
        requires = "erc20_eth_contract_address",
        requires = "amount_per_test",
        help = "Private key of the account to take funds from"
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
        requires = "amount_per_test",
        help = "Account address of the account to take funds from"
    )]
    pub account_address: Option<Felt>,

    #[arg(
        long,
        env,
        requires = "private_key",
        requires = "account_address",
        requires = "erc20_strk_contract_address",
        requires = "erc20_eth_contract_address",
        requires = "amount_per_test",
        help = "Account contract class hash"
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
        requires = "amount_per_test",
        help = "ERC20 STRK contract address"
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
        requires = "amount_per_test",
        help = "ERC20 ETH contract address"
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
        requires = "erc20_eth_contract_address",
        help = "Amount per test (preferably at least 0xfffffffffffffff)"
    )]
    pub amount_per_test: Option<Felt>,
}
