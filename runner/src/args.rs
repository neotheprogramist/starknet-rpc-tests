use clap::Parser;
use shared::account_balance::Version;
use starknet_crypto::FieldElement;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, short, env)]
    pub sender_address: FieldElement,

    #[arg(long, short, env)]
    pub vers: Version,

    #[arg(long, short, env)]
    pub url: Url,
}
