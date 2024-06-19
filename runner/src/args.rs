use clap::Parser;
use shared::account_balance::Version;
use starknet_crypto::FieldElement;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, short, env)]
    pub account_address: FieldElement,

    #[arg(long, short, env)]
    pub private_key: FieldElement,

    #[arg(long, short, env)]
    pub vers: Version,

    #[arg(long, short, env)]
    pub url: Url,

    #[arg(long,short,env, default_value_t = String::from("0x534e5f5345504f4c4941"))]
    pub chain_id: String,
}
